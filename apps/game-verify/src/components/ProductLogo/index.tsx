import logos from './images';

export type ProductLogoProps = {
  name: string;
  size: number;
  className?: string;
};

export function ProductLogo({ name, size = 40, className }: ProductLogoProps) {
  return <img src={logos[name]} width={size} height={size} className={className} />;
}