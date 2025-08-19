export type SvgSpriteProps = {
  name: string;
  size: number;
  className?: string;
};

export function SvgSprite({ name, size, className }: SvgSpriteProps) {
return <svg width={size} height={size} className={className}>
    <use xlinkHref={`#${name}`}></use>
  </svg>;
}