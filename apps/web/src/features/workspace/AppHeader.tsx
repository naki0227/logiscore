import { motion } from "framer-motion";
import { Icons } from "../../components/Icons";

interface AppHeaderProps {
  status: string;
  systemVersion: string | null;
  filename: string;
  projectFileCount: number | null;
}

export function AppHeader({
  status,
  systemVersion,
  filename,
  projectFileCount,
}: AppHeaderProps) {
  return (
    <header className="header">
      <motion.div
        initial={{ opacity: 0, x: -20 }}
        animate={{ opacity: 1, x: 0 }}
        className="logo"
      >
        <span className="logo-text">L O G I S C O R E</span>
        <span className="logo-version">v{systemVersion || "1.7"}</span>
      </motion.div>
      <div className="status-bar-container">
        <motion.div
          key={status}
          initial={{ opacity: 0, scale: 0.9 }}
          animate={{ opacity: 1, scale: 1 }}
          className="status-bar"
          role="status"
          aria-live="polite"
        >
          <div className="status-indicator" />
          {status}
        </motion.div>
      </div>
      <div className="header-meta">
        {projectFileCount !== null && (
          <div className="project-badge">
            <Icons.Node />
            <span>{projectFileCount} FILES</span>
          </div>
        )}
        <div className="filename-badge">
          <Icons.Check />
          <span>{filename}</span>
        </div>
      </div>
    </header>
  );
}
