import { useState } from "react";
import { motion } from "framer-motion";
import type { DreamscapeNode } from "../types/quest";
import {
  previewSiteTypes,
  rewardPreviewLabel,
  siteTypeIcon,
  siteTypeName,
} from "../atlas/atlas-generator";

const NODE_RADIUS_REGULAR = 28;
const NODE_RADIUS_NEXUS = 40;

interface AtlasNodeProps {
  node: DreamscapeNode;
  isNexus: boolean;
  onNodeClick: (nodeId: string) => void;
}

/** Renders a single node on the Dream Atlas graph. */
export function AtlasNode({ node, isNexus, onNodeClick }: AtlasNodeProps) {
  const [isHovered, setIsHovered] = useState(false);

  const radius = isNexus ? NODE_RADIUS_NEXUS : NODE_RADIUS_REGULAR;
  const isAvailable = node.status === "available";
  const isCompleted = node.status === "completed";
  const isUnavailable = node.status === "unavailable";

  const handleClick = () => {
    if (isAvailable) {
      onNodeClick(node.id);
    }
  };

  // Node opacity
  let opacity = 1;
  if (isUnavailable) {
    opacity = 0.35;
  } else if (isCompleted && !isNexus) {
    opacity = 0.6;
  }

  // Node color
  let fillColor = node.biomeColor;
  if (isNexus) {
    fillColor = "#7c3aed";
  } else if (isUnavailable) {
    fillColor = "#2d2040";
  }

  // Border color
  let strokeColor = node.biomeColor;
  if (isNexus) {
    strokeColor = "#a855f7";
  } else if (isAvailable) {
    strokeColor = "#e2e8f0";
  } else if (isUnavailable) {
    strokeColor = "#4a3860";
  } else {
    strokeColor = "#6b7280";
  }

  const previewSites = previewSiteTypes(node);

  const rewardLabel = node.sites
    .map((s) => rewardPreviewLabel(s))
    .find((label) => label !== null) ?? null;

  return (
    <g
      transform={`translate(${String(node.position.x)}, ${String(node.position.y)})`}
      style={{
        cursor: isAvailable ? "pointer" : "default",
        opacity,
      }}
      onClick={handleClick}
      onMouseEnter={() => { setIsHovered(true); }}
      onMouseLeave={() => { setIsHovered(false); }}
      role={isAvailable ? "button" : undefined}
      aria-label={
        isNexus
          ? "Nexus"
          : `${node.biomeName} dreamscape - ${node.status}`
      }
    >
      {/* Glow effect for available nodes */}
      {isAvailable && (
        <motion.circle
          cx={0}
          cy={0}
          r={radius + 8}
          fill="none"
          stroke={strokeColor}
          strokeWidth={2}
          initial={{ opacity: 0.3 }}
          animate={{ opacity: [0.3, 0.8, 0.3] }}
          transition={{
            duration: 2,
            repeat: Infinity,
            ease: "easeInOut",
          }}
        />
      )}

      {/* Main node circle */}
      <circle
        cx={0}
        cy={0}
        r={radius}
        fill={fillColor}
        fillOpacity={isNexus ? 0.4 : 0.25}
        stroke={strokeColor}
        strokeWidth={isNexus ? 3 : 2}
      />

      {/* Inner circle for nexus */}
      {isNexus && (
        <circle
          cx={0}
          cy={0}
          r={radius * 0.6}
          fill="#7c3aed"
          fillOpacity={0.6}
          stroke="#a855f7"
          strokeWidth={1}
        />
      )}

      {/* Checkmark for completed nodes */}
      {isCompleted && !isNexus && (
        <text
          x={0}
          y={4}
          textAnchor="middle"
          fill="#10b981"
          fontSize={radius * 0.8}
          fontWeight="bold"
          style={{ pointerEvents: "none" }}
        >
          {"\u2713"}
        </text>
      )}

      {/* Nexus label */}
      {isNexus && (
        <text
          x={0}
          y={5}
          textAnchor="middle"
          fill="#e2e8f0"
          fontSize={14}
          fontWeight="bold"
          style={{ pointerEvents: "none" }}
        >
          N
        </text>
      )}

      {/* Biome name below node (non-nexus) */}
      {!isNexus && (
        <text
          x={0}
          y={radius + 16}
          textAnchor="middle"
          fill="#e2e8f0"
          fontSize={10}
          opacity={isUnavailable ? 0.4 : 0.8}
          style={{ pointerEvents: "none" }}
        >
          {node.biomeName}
        </text>
      )}

      {/* Hover tooltip */}
      {isHovered && (isAvailable || isCompleted) && !isNexus && (
        <g transform={`translate(0, ${String(-(radius + 20))})`}>
          <rect
            x={-115}
            y={rewardLabel !== null ? -80 : -64}
            width={230}
            height={rewardLabel !== null ? 78 : 62}
            rx={8}
            fill="#1a1025"
            stroke={node.biomeColor}
            strokeWidth={1}
            opacity={0.95}
          />
          {/* Biome name in tooltip */}
          <text
            x={0}
            y={rewardLabel !== null ? -58 : -44}
            textAnchor="middle"
            fill={node.biomeColor}
            fontSize={12}
            fontWeight="bold"
          >
            {node.biomeName}
          </text>
          {/* Site icons */}
          <text
            x={0}
            y={rewardLabel !== null ? -36 : -22}
            textAnchor="middle"
            fontSize={18}
          >
            {previewSites.map((st, i) => (
              <tspan key={i} dx={i > 0 ? 8 : 0}>
                {siteTypeIcon(st)}
              </tspan>
            ))}
          </text>
          {/* Site names */}
          <text
            x={0}
            y={rewardLabel !== null ? -18 : -4}
            textAnchor="middle"
            fill="#e2e8f0"
            fontSize={8}
            opacity={0.7}
          >
            {previewSites.map((st) => siteTypeName(st)).join(" \u00B7 ")}
          </text>
          {/* Reward preview label */}
          {rewardLabel !== null && (
            <text
              x={0}
              y={-4}
              textAnchor="middle"
              fill="#fbbf24"
              fontSize={8}
              fontWeight="bold"
            >
              {rewardLabel}
            </text>
          )}
        </g>
      )}
    </g>
  );
}
