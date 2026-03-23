import { useRef, useEffect, useImperativeHandle, forwardRef } from 'react';

export interface VisualizerHandle {
  addNote: (note: number, velocity: number, duration: number, isBass: boolean) => void;
}

const Visualizer = forwardRef<VisualizerHandle>((_, ref) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const particlesRef = useRef<Array<{
    id: number;
    x: number;
    y: number;
    w: number;
    h: number;
    color: string;
    opacity: number;
    speed: number;
  }>>([]);

  const nextId = useRef(0);

  useImperativeHandle(ref, () => ({
    addNote(note: number, velocity: number, duration: number, isBass: boolean) {
      const canvas = canvasRef.current;
      if (!canvas) return;

      // MIDI note をマップ
      // ベースは低、メロディは中高域
      const normalized = (note - 24) / 84; 
      const x = (isBass ? 0.1 + (note % 12) / 100 : normalized) * canvas.width;
      
      const colors = isBass 
        ? ['#ffffff', '#6366f1'] // Bass: White/Indigo
        : ['#8b5cf6', '#ec4899', '#3b82f6', '#10b981']; // Melody: Colorful
        
      const color = colors[Math.floor(Math.random() * colors.length)];

      particlesRef.current.push({
        id: nextId.current++,
        x,
        y: -20,
        w: isBass ? 40 : Math.max(2, velocity * 25),
        h: isBass ? 150 : duration * 300,
        color,
        opacity: 1,
        speed: isBass ? 1.5 : 3 + Math.random() * 2,
      });
    }
  }));

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    let animationFrame: number;

    const render = () => {
      // 画面クリア（少し残像を残す）
      ctx.fillStyle = 'rgba(10, 10, 15, 0.2)';
      ctx.fillRect(0, 0, canvas.width, canvas.height);

      // 粒子更新
      particlesRef.current = particlesRef.current.filter(p => {
        p.y += p.speed;
        p.opacity -= 0.005;
        return p.y < canvas.height + 100 && p.opacity > 0;
      });

      // 描画
      for (const p of particlesRef.current) {
        ctx.save();
        ctx.globalAlpha = p.opacity;
        ctx.shadowBlur = 15;
        ctx.shadowColor = p.color;
        ctx.fillStyle = p.color;
        // 角丸の矩形
        const r = 4;
        ctx.beginPath();
        ctx.roundRect(p.x - p.w/2, p.y, p.w, p.h, r);
        ctx.fill();
        ctx.restore();
      }

      animationFrame = requestAnimationFrame(render);
    };

    render();
    return () => cancelAnimationFrame(animationFrame);
  }, []);

  return (
    <div className="visualizer-container">
      <canvas
        ref={canvasRef}
        width={800}
        height={400}
        className="visualizer-canvas"
      />
    </div>
  );
});

export default Visualizer;
