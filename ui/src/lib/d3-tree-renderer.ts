/**
 * D3.js horizontal tree renderer for ancestry visualization.
 *
 * Renders a left-to-right tree with:
 * - Curved connectors (linkHorizontal)
 * - Active path highlighting
 * - Collapsed sibling nodes with +N badges
 * - Selection and click handling
 */

import * as d3 from 'd3';
import type { HierarchyPointNode } from 'd3';
import type { Node } from './types';

// Layout constants
const NODE_WIDTH = 180;
const NODE_HEIGHT = 44;
const ROOT_NODE_HEIGHT = 64; // Extra height for goal text
const V_SPACING = 54; // Vertical gap between siblings
const H_SPACING = 220; // Horizontal gap parent→child
const PADDING = { top: 20, right: 40, bottom: 20, left: 20 };

// Visual constants
const STATUS_COLORS = {
  pass: { bg: '#dcfce7', text: '#166534' },
  fail: { bg: '#fee2e2', text: '#991b1b' },
  running: { bg: '#dbeafe', text: '#1e40af' },
  pending: { bg: '#f1f5f9', text: '#64748b' },
} as const;

// Internal hierarchy node type
interface TreeNode {
  id: string;
  title: string;
  goal?: string;
  passes: boolean;
  attempts: number;
  maxAttempts: number;
  isCollapsed: boolean;
  collapsedCount: number; // Direct children count for +N badge
  isRoot: boolean;
  children?: TreeNode[];
}

export interface TreeRendererOptions {
  tree: Node;
  activePath: Set<string>;
  selectedNodeId: string | null;
  onNodeClick: (node: Node) => void;
}

/**
 * Creates a tree renderer attached to an SVG element.
 * Returns a destroy function for cleanup.
 */
export function createTreeRenderer(
  svg: SVGSVGElement,
  options: TreeRendererOptions
): { destroy: () => void } {
  const { tree, activePath, selectedNodeId, onNodeClick } = options;

  // Clear any existing content
  const svgSelection = d3.select(svg);
  svgSelection.selectAll('*').remove();

  // Build hierarchy with collapse logic
  const root = buildHierarchy(tree, activePath, true);
  const hierarchy = d3.hierarchy(root);

  // Compute layout using nodeSize for consistent spacing
  const treeLayout = d3.tree<TreeNode>().nodeSize([V_SPACING, H_SPACING]);
  const layoutRoot = treeLayout(hierarchy);

  // Calculate bounds for viewBox
  const bounds = computeBounds(layoutRoot);

  // Setup SVG with proper viewBox
  const g = setupSvg(svgSelection, bounds);

  // Render links first (below nodes)
  renderLinks(g, layoutRoot, activePath);

  // Render nodes
  renderNodes(g, layoutRoot, activePath, selectedNodeId, tree, onNodeClick);

  return {
    destroy: () => {
      svgSelection.selectAll('*').remove();
    },
  };
}

/**
 * Build a D3-compatible hierarchy from the Node tree.
 * Collapses off-path siblings (shows them as single nodes with +N badge).
 */
function buildHierarchy(
  node: Node,
  activePath: Set<string>,
  isRoot: boolean
): TreeNode {
  const isOnPath = activePath.has(node.id);

  const treeNode: TreeNode = {
    id: node.id,
    title: node.title,
    goal: isRoot ? node.goal : undefined,
    passes: node.passes,
    attempts: node.attempts,
    maxAttempts: node.max_attempts,
    isCollapsed: false,
    collapsedCount: 0,
    isRoot,
  };

  if (node.children.length === 0) {
    return treeNode;
  }

  // Process children
  const children: TreeNode[] = [];

  for (const child of node.children) {
    const childOnPath = activePath.has(child.id);

    if (childOnPath) {
      // Recurse into on-path children
      children.push(buildHierarchy(child, activePath, false));
    } else {
      // Collapsed sibling - show as single node
      children.push({
        id: child.id,
        title: child.title,
        passes: child.passes,
        attempts: child.attempts,
        maxAttempts: child.max_attempts,
        isCollapsed: true,
        collapsedCount: child.children.length,
        isRoot: false,
      });
    }
  }

  treeNode.children = children;
  return treeNode;
}

/**
 * Compute the bounding box of all nodes.
 */
function computeBounds(root: HierarchyPointNode<TreeNode>): {
  minX: number;
  maxX: number;
  minY: number;
  maxY: number;
} {
  let minX = Infinity;
  let maxX = -Infinity;
  let minY = Infinity;
  let maxY = -Infinity;

  root.each((node) => {
    // Note: d3.tree uses x for vertical position, y for horizontal
    // We swap them for horizontal layout
    minX = Math.min(minX, node.x);
    maxX = Math.max(maxX, node.x);
    minY = Math.min(minY, node.y);
    maxY = Math.max(maxY, node.y);
  });

  return { minX, maxX, minY, maxY };
}

/**
 * Setup SVG with proper viewBox and create main group.
 */
function setupSvg(
  svg: d3.Selection<SVGSVGElement, unknown, null, undefined>,
  bounds: { minX: number; maxX: number; minY: number; maxY: number }
): d3.Selection<SVGGElement, unknown, null, undefined> {
  // Width/height based on bounds (swap x/y for horizontal layout)
  const width =
    bounds.maxY - bounds.minY + NODE_WIDTH + PADDING.left + PADDING.right;
  const height =
    bounds.maxX - bounds.minX + ROOT_NODE_HEIGHT + PADDING.top + PADDING.bottom;

  // Set viewBox for responsive scaling
  const viewBoxX = bounds.minY - PADDING.left;
  const viewBoxY = bounds.minX - NODE_HEIGHT / 2 - PADDING.top;

  svg
    .attr('width', '100%')
    .attr('height', '100%')
    .attr('viewBox', `${viewBoxX} ${viewBoxY} ${width} ${height}`)
    .attr('preserveAspectRatio', 'xMidYMid meet');

  // Create main group (no transform needed, nodes positioned absolutely)
  return svg.append('g');
}

/**
 * Render link paths between nodes.
 */
function renderLinks(
  g: d3.Selection<SVGGElement, unknown, null, undefined>,
  root: HierarchyPointNode<TreeNode>,
  activePath: Set<string>
): void {
  // Horizontal link generator (swap x/y)
  const linkGen = d3
    .linkHorizontal<
      d3.HierarchyPointLink<TreeNode>,
      HierarchyPointNode<TreeNode>
    >()
    .x((d) => d.y)
    .y((d) => d.x);

  const links = root.links();

  g.selectAll('path.link')
    .data(links)
    .join('path')
    .attr('class', (d) => {
      const sourceOnPath = activePath.has(d.source.data.id);
      const targetOnPath = activePath.has(d.target.data.id);
      return `link ${sourceOnPath && targetOnPath ? 'on-path' : ''}`;
    })
    .attr('d', (d) => {
      // Adjust source/target positions to connect from node edges
      const source = {
        ...d.source,
        y: d.source.y + NODE_WIDTH, // Right edge of parent
      };
      const target = {
        ...d.target,
        y: d.target.y, // Left edge of child
      };
      return linkGen({ source, target } as d3.HierarchyPointLink<TreeNode>);
    })
    .attr('fill', 'none')
    .attr('stroke', (d) => {
      const sourceOnPath = activePath.has(d.source.data.id);
      const targetOnPath = activePath.has(d.target.data.id);
      return sourceOnPath && targetOnPath ? '#3b82f6' : '#e2e8f0';
    })
    .attr('stroke-width', 2);
}

/**
 * Render node rectangles and labels.
 */
function renderNodes(
  g: d3.Selection<SVGGElement, unknown, null, undefined>,
  root: HierarchyPointNode<TreeNode>,
  activePath: Set<string>,
  selectedNodeId: string | null,
  originalTree: Node,
  onNodeClick: (node: Node) => void
): void {
  const nodes = root.descendants();

  // Create node groups
  const nodeGroups = g
    .selectAll('g.node')
    .data(nodes)
    .join('g')
    .attr('class', 'node')
    .attr('data-node-id', (d) => d.data.id)
    .attr('transform', (d) => `translate(${d.y}, ${d.x})`)
    .style('cursor', 'pointer')
    .on('click', (event, d) => {
      event.stopPropagation();
      const originalNode = findNodeById(originalTree, d.data.id);
      if (originalNode) {
        onNodeClick(originalNode);
      }
    });

  // Render each node
  nodeGroups.each(function (d) {
    const nodeG = d3.select(this);
    const data = d.data;
    const isOnPath = activePath.has(data.id);
    const isSelected = selectedNodeId === data.id;
    const hasChildren = d.children && d.children.length > 0;
    const isActiveLeaf = isOnPath && !hasChildren && !data.isCollapsed;
    const nodeHeight = data.isRoot && data.goal ? ROOT_NODE_HEIGHT : NODE_HEIGHT;

    // Determine fill and stroke colors
    let fill = '#f8fafc';
    let stroke = '#e2e8f0';

    if (isActiveLeaf) {
      fill = '#dbeafe';
      stroke = '#3b82f6';
    } else if (isSelected) {
      fill = '#e0f2fe';
      stroke = '#7dd3fc';
    } else if (isOnPath) {
      stroke = '#93c5fd';
    }

    if (data.isCollapsed) {
      nodeG.attr('opacity', 0.7);
    }

    // Node rectangle
    nodeG
      .append('rect')
      .attr('class', 'node-rect')
      .attr('x', 0)
      .attr('y', -nodeHeight / 2)
      .attr('width', NODE_WIDTH)
      .attr('height', nodeHeight)
      .attr('rx', 6)
      .attr('fill', fill)
      .attr('stroke', stroke)
      .attr('stroke-width', isActiveLeaf ? 2 : 1);

    // Active leaf glow
    if (isActiveLeaf) {
      nodeG
        .insert('rect', ':first-child')
        .attr('class', 'node-glow')
        .attr('x', -2)
        .attr('y', -nodeHeight / 2 - 2)
        .attr('width', NODE_WIDTH + 4)
        .attr('height', nodeHeight + 4)
        .attr('rx', 8)
        .attr('fill', 'none')
        .attr('stroke', 'rgba(59, 130, 246, 0.2)')
        .attr('stroke-width', 4);
    }

    // Title text
    const titleY = data.isRoot && data.goal ? -8 : 0;
    nodeG
      .append('text')
      .attr('class', 'node-title')
      .attr('x', 10)
      .attr('y', titleY)
      .attr('dy', '0.35em')
      .attr('font-size', '14px')
      .attr('font-weight', 500)
      .attr('fill', '#1e293b')
      .text(truncateText(data.title, 16));

    // Goal text for root node
    if (data.isRoot && data.goal) {
      nodeG
        .append('text')
        .attr('class', 'node-goal')
        .attr('x', 10)
        .attr('y', 12)
        .attr('dy', '0.35em')
        .attr('font-size', '12px')
        .attr('fill', '#64748b')
        .text(truncateText(data.goal, 22));
    }

    // Status badge
    const status = getStatus(data);
    const statusLabel = getStatusLabel(data);
    const colors = STATUS_COLORS[status];

    const badgeWidth = statusLabel.length * 6 + 12;
    const badgeX = NODE_WIDTH - badgeWidth - 8;

    nodeG
      .append('rect')
      .attr('class', 'status-badge-bg')
      .attr('x', badgeX)
      .attr('y', titleY - 8)
      .attr('width', badgeWidth)
      .attr('height', 16)
      .attr('rx', 8)
      .attr('fill', colors.bg);

    nodeG
      .append('text')
      .attr('class', 'status-badge-text')
      .attr('x', badgeX + badgeWidth / 2)
      .attr('y', titleY)
      .attr('dy', '0.35em')
      .attr('text-anchor', 'middle')
      .attr('font-size', '10px')
      .attr('font-weight', 500)
      .attr('fill', colors.text)
      .text(statusLabel);

    // Collapsed badge (+N)
    if (data.isCollapsed && data.collapsedCount > 0) {
      const collapseBadgeX = NODE_WIDTH - 28;
      const collapseBadgeY = titleY + (data.isRoot && data.goal ? 20 : 16);

      nodeG
        .append('rect')
        .attr('class', 'collapse-badge-bg')
        .attr('x', collapseBadgeX)
        .attr('y', collapseBadgeY - 8)
        .attr('width', 24)
        .attr('height', 16)
        .attr('rx', 4)
        .attr('fill', '#e2e8f0');

      nodeG
        .append('text')
        .attr('class', 'collapse-badge-text')
        .attr('x', collapseBadgeX + 12)
        .attr('y', collapseBadgeY)
        .attr('dy', '0.35em')
        .attr('text-anchor', 'middle')
        .attr('font-size', '10px')
        .attr('fill', '#64748b')
        .text(`+${data.collapsedCount}`);
    }
  });
}

/**
 * Helper: find a node by ID in the original tree.
 */
function findNodeById(tree: Node, id: string): Node | null {
  if (tree.id === id) return tree;
  for (const child of tree.children) {
    const found = findNodeById(child, id);
    if (found) return found;
  }
  return null;
}

/**
 * Helper: get status string from node data.
 */
function getStatus(node: TreeNode): 'pass' | 'fail' | 'running' | 'pending' {
  if (node.passes) return 'pass';
  if (node.attempts >= node.maxAttempts) return 'fail';
  if (node.attempts > 0) return 'running';
  return 'pending';
}

/**
 * Helper: get status label for display.
 */
function getStatusLabel(node: TreeNode): string {
  if (node.passes) return 'pass';
  if (node.attempts >= node.maxAttempts) return 'fail';
  if (node.attempts > 0) return `${node.attempts}/${node.maxAttempts}`;
  return 'pending';
}

/**
 * Helper: truncate text with ellipsis.
 */
function truncateText(text: string, maxLength: number): string {
  if (text.length <= maxLength) return text;
  return text.slice(0, maxLength - 1) + '\u2026';
}
