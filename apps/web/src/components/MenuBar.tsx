import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { handleMenuAction } from '../commands/menuActions';
import { getMenuBarDef, type MenuItemDef } from '../menus/menuDefinition';
import './MenuBar.css';

function MenuDropdown({
  items,
  onAction,
  onClose,
}: {
  items: MenuItemDef[];
  onAction: (id: string) => void;
  onClose: () => void;
}) {
  return (
    <div className="menu-dropdown" role="menu">
      {items.map((item, index) => {
        if (item.type === 'separator') {
          return <div key={`separator-${index}`} className="menu-dropdown__separator" />;
        }

        return (
          <button
            key={item.id}
            type="button"
            className="menu-dropdown__item"
            role="menuitem"
            onClick={() => {
              onAction(item.id);
              onClose();
            }}
          >
            <span className="menu-dropdown__item-label">{item.label}</span>
            {item.shortcut && (
              <span className="menu-dropdown__item-shortcut">{item.shortcut}</span>
            )}
          </button>
        );
      })}
    </div>
  );
}

export function MenuBar() {
  const [openMenu, setOpenMenu] = useState<number | null>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const menuDef = useMemo(() => getMenuBarDef(), []);

  const closeMenu = useCallback(() => {
    setOpenMenu(null);
  }, []);

  const runAction = useCallback((id: string) => {
    void handleMenuAction(id);
  }, []);

  useEffect(() => {
    if (openMenu === null) return undefined;

    const onClickOutside = (event: MouseEvent) => {
      if (!menuRef.current?.contains(event.target as Node)) {
        setOpenMenu(null);
      }
    };
    const onEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') setOpenMenu(null);
    };

    document.addEventListener('mousedown', onClickOutside);
    document.addEventListener('keydown', onEscape);
    return () => {
      document.removeEventListener('mousedown', onClickOutside);
      document.removeEventListener('keydown', onEscape);
    };
  }, [openMenu]);

  return (
    <div className="menubar" ref={menuRef}>
      <div className="menubar__menus">
        {menuDef.map((menu, index) => (
          <div key={menu.label} className="menubar__menu-wrapper">
            <button
              type="button"
              className={`menubar__trigger ${
                openMenu === index ? 'menubar__trigger--active' : ''
              }`}
              aria-haspopup="menu"
              aria-expanded={openMenu === index}
              onClick={() => setOpenMenu(openMenu === index ? null : index)}
              onMouseEnter={() => {
                if (openMenu !== null) setOpenMenu(index);
              }}
            >
              {menu.label}
            </button>
            {openMenu === index && (
              <MenuDropdown items={menu.items} onAction={runAction} onClose={closeMenu} />
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
