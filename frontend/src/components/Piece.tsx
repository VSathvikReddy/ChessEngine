/** Unicode chess piece renderer. Uses SVG text for crisp rendering at any size. */

const GLYPHS: Record<string, Record<"w" | "b", string>> = {
  K: { w: "♔", b: "♚" },
  Q: { w: "♕", b: "♛" },
  R: { w: "♖", b: "♜" },
  B: { w: "♗", b: "♝" },
  N: { w: "♘", b: "♞" },
  P: { w: "♙", b: "♟" },
};

interface PieceProps {
  type: string;    // K Q R B N P
  color: "w" | "b";
  size: number;
}

export function Piece({ type, color, size }: PieceProps) {
  const glyph = GLYPHS[type]?.[color] ?? "?";

  return (
    <text
      x="50%"
      y="54%"
      dominantBaseline="middle"
      textAnchor="middle"
      fontSize={size * 0.72}
      fill={color === "w" ? "#f0d9b5" : "#1a1a2e"}
      stroke={color === "w" ? "#b58863" : "#f0d9b5"}
      strokeWidth={size * 0.015}
      style={{ userSelect: "none", cursor: "pointer", fontFamily: "serif" }}
    >
      {glyph}
    </text>
  );
}
