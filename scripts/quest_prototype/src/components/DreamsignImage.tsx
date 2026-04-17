import { useState } from "react";

export interface DreamsignImageProps {
  name: string;
  imageName?: string;
  imageAlt?: string;
  className?: string;
  frameClassName?: string;
  placeholderClassName?: string;
  imgClassName?: string;
  isBane?: boolean;
}

/** Returns the public asset URL for a Dreamsign image file. */
export function dreamsignImageUrl(imageName?: string): string | null {
  if (imageName === undefined || imageName.trim().length === 0) {
    return null;
  }

  return `/dreamsigns/${encodeURIComponent(imageName)}`;
}

/** Renders a Dreamsign image with a resilient placeholder fallback. */
export function DreamsignImage({
  name,
  imageName,
  imageAlt,
  className = "h-16 w-16",
  frameClassName = "",
  placeholderClassName = "",
  imgClassName = "",
  isBane = false,
}: DreamsignImageProps) {
  const [failed, setFailed] = useState(false);
  const imageUrl = failed ? null : dreamsignImageUrl(imageName);

  return (
    <div
      className={`overflow-hidden rounded-2xl ${className} ${frameClassName}`.trim()}
      style={{
        background:
          "radial-gradient(circle at top, rgba(196, 181, 253, 0.22), rgba(15, 10, 24, 0.95) 68%)",
      }}
    >
      {imageUrl === null ? (
        <div
          className={`flex h-full w-full items-center justify-center text-2xl ${placeholderClassName}`.trim()}
          aria-label={imageAlt ?? `${name} Dreamsign`}
        >
          {isBane ? "\u2620" : "\u2726"}
        </div>
      ) : (
        <img
          src={imageUrl}
          alt={imageAlt ?? `${name} Dreamsign`}
          className={`h-full w-full object-cover ${imgClassName}`.trim()}
          loading="lazy"
          onError={() => setFailed(true)}
        />
      )}
    </div>
  );
}
