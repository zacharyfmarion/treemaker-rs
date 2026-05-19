import * as RadixSelect from '@radix-ui/react-select';
import { forwardRef, type ComponentPropsWithoutRef } from 'react';
import { ChevronDown } from 'lucide-react';
import { CONTROL_RADIUS_CLASS } from './controlStyles';

export const Select = RadixSelect.Root;
export const SelectValue = RadixSelect.Value;

export const SelectTrigger = forwardRef<
  HTMLButtonElement,
  ComponentPropsWithoutRef<typeof RadixSelect.Trigger>
>(({ children, className = '', ...props }, ref) => (
  <RadixSelect.Trigger
    ref={ref}
    className={`select-trigger ${CONTROL_RADIUS_CLASS} ${className}`.trim()}
    {...props}
  >
    {children}
    <RadixSelect.Icon asChild>
      <ChevronDown size={14} />
    </RadixSelect.Icon>
  </RadixSelect.Trigger>
));

SelectTrigger.displayName = 'SelectTrigger';

export const SelectContent = forwardRef<
  HTMLDivElement,
  ComponentPropsWithoutRef<typeof RadixSelect.Content>
>(({ children, className = '', ...props }, ref) => (
  <RadixSelect.Portal>
    <RadixSelect.Content
      ref={ref}
      className={`select-content ${className}`.trim()}
      position="popper"
      sideOffset={4}
      {...props}
    >
      <RadixSelect.Viewport className="select-content__viewport">{children}</RadixSelect.Viewport>
    </RadixSelect.Content>
  </RadixSelect.Portal>
));

SelectContent.displayName = 'SelectContent';

export const SelectItem = forwardRef<
  HTMLDivElement,
  ComponentPropsWithoutRef<typeof RadixSelect.Item>
>(({ children, className = '', ...props }, ref) => (
  <RadixSelect.Item ref={ref} className={`select-item ${className}`.trim()} {...props}>
    <RadixSelect.ItemText>{children}</RadixSelect.ItemText>
  </RadixSelect.Item>
));

SelectItem.displayName = 'SelectItem';
