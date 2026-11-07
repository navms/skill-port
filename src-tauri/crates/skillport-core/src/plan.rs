//! 同步计划的短期存储：TTL、取消标记。

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::error::{AppError, AppResult};
use super::model::SyncPlan;
use super::util::lock;

/// 计划有效期：超过后必须重新计算。
pub const PLAN_TTL: Duration = Duration::from_secs(300);

struct PlanHandle {
    plan: SyncPlan,
    cancel: Arc<AtomicBool>,
    created: Instant,
}

/// 进程内计划表。
#[derive(Default)]
pub struct PlanStore {
    inner: Mutex<HashMap<String, PlanHandle>>,
}

impl PlanStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// 保存计划，返回申请到的取消标记。
    pub fn insert(&self, plan: SyncPlan) -> AppResult<Arc<AtomicBool>> {
        let cancel = Arc::new(AtomicBool::new(false));
        let handle = PlanHandle {
            plan,
            cancel: cancel.clone(),
            created: Instant::now(),
        };
        let mut map = lock(&self.inner)?;
        map.retain(|_, existing| existing.created.elapsed() <= PLAN_TTL);
        map.insert(handle.plan.id.clone(), handle);
        Ok(cancel)
    }

    /// 开始执行：校验存在与 TTL，返回计划与取消标记；不删除条目，
    /// 以便执行期间仍能收到 `sync_cancel`。
    pub fn begin_apply(&self, id: &str) -> AppResult<(SyncPlan, Arc<AtomicBool>)> {
        let expired = {
            let map = lock(&self.inner)?;
            match map.get(id) {
                None => true,
                Some(handle) => handle.created.elapsed() > PLAN_TTL,
            }
        };
        if expired {
            self.finish(id)?;
            return Err(AppError::PlanExpired(id.to_string()));
        }
        let map = lock(&self.inner)?;
        let handle = map
            .get(id)
            .ok_or_else(|| AppError::PlanExpired(id.to_string()))?;
        Ok((handle.plan.clone(), handle.cancel.clone()))
    }

    /// 请求取消；正在执行的计划会在下一步之前停下。
    pub fn cancel(&self, id: &str) -> AppResult<()> {
        let map = lock(&self.inner)?;
        let handle = map
            .get(id)
            .ok_or_else(|| AppError::PlanExpired(id.to_string()))?;
        handle.cancel.store(true, Ordering::SeqCst);
        Ok(())
    }

    /// 结束并移除计划。
    pub fn finish(&self, id: &str) -> AppResult<()> {
        let mut map = lock(&self.inner)?;
        map.remove(id);
        Ok(())
    }

    /// 清理过期计划。
    pub fn purge_expired(&self) -> AppResult<()> {
        let mut map = lock(&self.inner)?;
        map.retain(|_, handle| handle.created.elapsed() <= PLAN_TTL);
        Ok(())
    }
}
