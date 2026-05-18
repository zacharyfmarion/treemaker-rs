import * as RadixTooltip from '@radix-ui/react-tooltip';
import type { ComponentPropsWithoutRef } from 'react';

export const TooltipProvider = RadixTooltip.Provider;
export const Tooltip = RadixTooltip.Root;
export const TooltipTrigger = RadixTooltip.Trigger;

export function TooltipContent({
  side = 'top',
  sideOffset = 6,
  className = '',
  ...props
}: ComponentPropsWithoutRef<typeof RadixTooltip.Content>) {
  return (
    <RadixTooltip.Portal>
      <RadixTooltip.Content
        side={side}
        sideOffset={sideOffset}
        className={`tooltip-content ${className}`.trim()}
        {...props}
      />
    </RadixTooltip.Portal>
  );
}
