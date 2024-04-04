import { useTheme } from '@/components/misc/ThemeProvider.tsx';
import { useEffect, useState } from 'react';
import { Sun, Moon } from 'lucide-react';

export function ThemeToggle() {
  const { theme, setTheme } = useTheme();

  function toggleTheme() {
    setTheme(theme === 'dark' ? 'light' : 'dark');
  }

  const [bgc, setBgc] = useState('rgb(15, 23, 42)');
  const [fgc, setFgc] = useState('rgb(15, 23, 42)');
  const [visibilityClass, setVisibilityClass] = useState('');

  useEffect(() => {
    if (theme === 'light') {
      setBgc('rgb(15, 23, 42)');
      setFgc('rgb(255, 255, 255)');
      return;
    }
    setBgc('rgb(255, 255, 255)');
    setFgc('rgb(15, 23, 42)');
  }, [theme]);

  useEffect(() => {
    const handleVisibilityChange = () => {
      if (document.hidden || document.visibilityState === 'hidden') {
        setVisibilityClass('hidden');
        return;
      }
      setVisibilityClass('');
    };

    const handleBlur = () => setVisibilityClass('hidden');
    const handleFocus = () => setVisibilityClass('');

    window.addEventListener('visibilitychange', handleVisibilityChange);
    window.addEventListener('blur', handleBlur);
    window.addEventListener('focus', handleFocus);

    return () => {
      window.removeEventListener('visibilitychange', handleVisibilityChange);
      window.removeEventListener('blur', handleBlur);
      window.removeEventListener('focus', handleFocus);
    };
  }, []);

  return (
    <div
      className={
        'group absolute top-2 h-3 w-3 flex rounded-full items-center justify-center bg-gray-800 text-white ' +
        visibilityClass
      }
      style={{ left: '68px', backgroundColor: bgc }}
      onClick={() => {
        toggleTheme();
      }}
    >
      {theme === 'dark' ? (
        <Sun size={8} color={fgc} className="opacity-0 group-hover:opacity-100 duration-100" />
      ) : (
        <Moon size={8} color={fgc} className="opacity-0 group-hover:opacity-100 duration-100" />
      )}
    </div>
  );
}
