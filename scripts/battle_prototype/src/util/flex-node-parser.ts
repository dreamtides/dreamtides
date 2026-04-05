import type { FlexNode, GameAction, NodeType } from "../types/battle";

export interface ExtractedButton {
  label: string;
  action: GameAction;
}

export interface ExtractedOverlay {
  texts: string[];
  buttons: ExtractedButton[];
}

/** Strip Unity rich text tags, icon font characters, and trailing icon glyphs */
function stripRichText(text: string): string {
  return text
    .replace(/<\/?[^>]+>/gi, "")
    // Strip icon font characters (PUA, Arabic Presentation Forms)
    .replace(/[\uE000-\uF8FF\uFB50-\uFDFF\uFE70-\uFEFF]/g, "")
    // Strip trailing standalone CJK/symbol characters used as Unity icons
    // (e.g. "Choose cards to discard. 粒" → "Choose cards to discard.")
    .replace(/\s+[\u2E80-\u9FFF\uF900-\uFAFF]$/g, "")
    .trim();
}

/** Check if a label is only icon/symbol characters with no readable text */
function isIconOnlyLabel(text: string): boolean {
  const stripped = text.trim();
  return stripped.length <= 2 && /^[\u2E80-\u9FFF\uF900-\uFAFF]$/.test(stripped);
}

function getTextFromNodeType(nodeType: NodeType): string | null {
  if ("Text" in nodeType) return stripRichText(nodeType.Text.label);
  if ("TypewriterTextNode" in nodeType) return stripRichText(nodeType.TypewriterTextNode.label);
  return null;
}

function extractFromNode(
  node: FlexNode,
  result: ExtractedOverlay,
): void {
  // Extract text from this node
  if (node.node_type) {
    const text = getTextFromNodeType(node.node_type);
    if (text && text.trim()) {
      result.texts.push(text.trim());
    }
  }

  // Extract click handler as a button
  if (node.event_handlers?.on_click) {
    // Find label from child TextNodes
    let label = "";
    for (const child of node.children) {
      if (child.node_type) {
        const text = getTextFromNodeType(child.node_type);
        if (text) {
          label = text;
          break;
        }
      }
    }
    if (label && !isIconOnlyLabel(label)) {
      result.buttons.push({
        label,
        action: node.event_handlers.on_click,
      });
      // Don't recurse into button children (already extracted label)
      return;
    }
  }

  // Recurse into children
  for (const child of node.children) {
    extractFromNode(child, result);
  }
}

export function extractOverlayContent(
  overlay: FlexNode,
): ExtractedOverlay | null {
  const result: ExtractedOverlay = { texts: [], buttons: [] };
  extractFromNode(overlay, result);
  if (result.texts.length === 0 && result.buttons.length === 0) return null;
  return result;
}
