import { forwardRef, type ButtonHTMLAttributes } from 'react';
import { cva, type VariantProps } from 'class-variance-authority';
import { CONTROL_RADIUS_CLASS, CONTROL_SIZE_CLASSES } from './controlStyles';

const button = cva(['ui-button', CONTROL_RADIUS_CLASS].join(' '), {
  variants: {
    variant: {
      primary: 'ui-button--primary',
      secondary: 'ui-button--secondary',
      danger: 'ui-button--danger',
      ghost: 'ui-button--ghost',
    },
    size: CONTROL_SIZE_CLASSES,
  },
  defaultVariants: {
    variant: 'secondary',
    size: 'md',
  },
});

export interface ButtonProps
  extends ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof button> {
  isActive?: boolean;
}

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ variant, size, isActive, className = '', type = 'button', ...props }, ref) => (
    <button
      ref={ref}
      type={type}
      className={button({ variant, size, className })}
      data-active={isActive || undefined}
      {...props}
    />
  )
);

Button.displayName = 'Button';
