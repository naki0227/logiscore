import React from 'react';

interface PanelProps {
  title: string;
  headerAction?: React.ReactNode;
  children: React.ReactNode;
  className?: string;
  verified?: boolean;
}

const Panel: React.FC<PanelProps> = ({ title, headerAction, children, className = '', verified }) => {
  return (
    <section className={`panel ${className}`}>
      <div className="panel-header">
        <span className="panel-title">{title}</span>
        {verified && <span className="verified-badge">✅ VERIFIED</span>}
        {headerAction}
      </div>
      <div className="panel-content">
        {children}
      </div>
    </section>
  );
};

export default Panel;
