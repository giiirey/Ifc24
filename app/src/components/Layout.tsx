interface LayoutProps {
  children: React.ReactNode;
}

function Layout({ children }: LayoutProps) {
  return (
    <div style={{ display: 'flex', height: '100vh', width: 'calc(100vw - var(--panel-width-right))', overflow: 'hidden' }}>
      {children}
    </div>
  );
}

export default Layout;
