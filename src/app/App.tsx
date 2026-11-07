import { HashRouter } from 'react-router-dom'
import { AppShell } from './AppShell'
import { Providers } from './Providers'

export function App() {
  return (
    <HashRouter>
      <Providers>
        <AppShell />
      </Providers>
    </HashRouter>
  )
}
