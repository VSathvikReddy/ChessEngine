import { useState, useCallback } from "react";
import { Piece } from "./Piece";
import { parseFen, parsePiece, coordsToSq } from "../types/game";
import type { ParsedFen } from "../types/game";

interface BoardProps {
  fen: string;
  legalMoves: [string, string][];
  lastMove: [string, string] | null;
  aiMove: [string, string] | null;
  flipped: boolean;
  onMove: (from: string, to: string) => void;
}

const LIGHT = "#f0d9b5";
const DARK = "#b58863";
const SELECTED = "#f6f669cc";
const LEGAL_DOT = "#00000033";
const LAST_MOVE = "#cdd16e99";
const AI_MOVE_COLOR = "#7fc97f99";

const CELL = 80;
const BOARD = CELL * 8;

export function Board({ fen, legalMoves, lastMove, aiMove, flipped, onMove }: BoardProps) {
  const [selected, setSelected] = useState<string | null>(null);
  const parsed: ParsedFen = parseFen(fen);

  const legalTargets = selected
    ? legalMoves.filter(([from]) => from === selected).map(([, to]) => to)
    : [];

  const handleSquareClick = useCallback(
    (sq: string) => {
      if (selected) {
        if (legalTargets.includes(sq)) {
          onMove(selected, sq);
          setSelected(null);
        } else {
          // Re-select if clicking our own piece
          const { file, rank } = { file: sq.charCodeAt(0) - 97, rank: parseInt(sq[1]) - 1 };
          const piece = parsed.squares[rank]?.[file];
          if (piece) {
            const { color } = parsePiece(piece);
            if (color === parsed.sideToMove) {
              setSelected(sq);
              return;
            }
          }
          setSelected(null);
        }
      } else {
        const { file, rank } = { file: sq.charCodeAt(0) - 97, rank: parseInt(sq[1]) - 1 };
        const piece = parsed.squares[rank]?.[file];
        if (piece) {
          const { color } = parsePiece(piece);
          if (color === parsed.sideToMove) {
            setSelected(sq);
          }
        }
      }
    },
    [selected, legalTargets, parsed, onMove]
  );

  const ranks = flipped ? [0, 1, 2, 3, 4, 5, 6, 7] : [7, 6, 5, 4, 3, 2, 1, 0];
  const files = flipped ? [7, 6, 5, 4, 3, 2, 1, 0] : [0, 1, 2, 3, 4, 5, 6, 7];

  return (
    <svg
      width={BOARD + 24}
      height={BOARD + 24}
      viewBox={`-24 0 ${BOARD + 24} ${BOARD + 24}`}
      style={{ display: "block" }}
    >
      {/* Rank labels */}
      {ranks.map((rank, row) => (
        <text
          key={rank}
          x={-16}
          y={row * CELL + CELL / 2 + 5}
          fontSize={12}
          fill="#999"
          textAnchor="middle"
          fontFamily="monospace"
        >
          {rank + 1}
        </text>
      ))}

      {/* File labels */}
      {files.map((file, col) => (
        <text
          key={file}
          x={col * CELL + CELL / 2}
          y={BOARD + 18}
          fontSize={12}
          fill="#999"
          textAnchor="middle"
          fontFamily="monospace"
        >
          {String.fromCharCode(97 + file)}
        </text>
      ))}

      {/* Squares */}
      {ranks.map((rank, row) =>
        files.map((file, col) => {
          const sq = coordsToSq(file, rank);
          const isLight = (file + rank) % 2 !== 0;
          const isSelected = sq === selected;
          const isLegal = legalTargets.includes(sq);
          const isLastMove = lastMove && (lastMove[0] === sq || lastMove[1] === sq);
          const isAiMove = aiMove && (aiMove[0] === sq || aiMove[1] === sq);
          const piece = parsed.squares[rank]?.[file];

          let fill = isLight ? LIGHT : DARK;
          if (isLastMove) fill = LAST_MOVE;
          if (isAiMove) fill = AI_MOVE_COLOR;
          if (isSelected) fill = SELECTED;

          return (
            <g
              key={sq}
              transform={`translate(${col * CELL}, ${row * CELL})`}
              onClick={() => handleSquareClick(sq)}
              style={{ cursor: "pointer" }}
            >
              <rect width={CELL} height={CELL} fill={fill} />

              {/* Legal move indicator */}
              {isLegal && (
                piece ? (
                  <rect
                    width={CELL}
                    height={CELL}
                    fill="none"
                    stroke="#3d8b37"
                    strokeWidth={4}
                  />
                ) : (
                  <circle
                    cx={CELL / 2}
                    cy={CELL / 2}
                    r={CELL * 0.15}
                    fill={LEGAL_DOT}
                  />
                )
              )}

              {piece && (
                <svg width={CELL} height={CELL}>
                  <Piece
                    type={parsePiece(piece).type}
                    color={parsePiece(piece).color}
                    size={CELL}
                  />
                </svg>
              )}
            </g>
          );
        })
      )}
    </svg>
  );
}
