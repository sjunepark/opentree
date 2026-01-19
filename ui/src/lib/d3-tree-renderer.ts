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
const NODE_WIDTH = 200;
const NODE_HEIGHT = 60; // Height for 2-line wrapped titles
const ROOT_NODE_HEIGHT = 80; // Extra height for goal text
const V_SPACING = 70; // Vertical gap between siblings
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
  expandedPath: Set<string>;  // Controls which nodes are expanded (visible)
  highlightedPath: Set<string>;  // Controls which nodes are highlighted (blue)
  selectedNodeId: string | null;
  onNodeClick: (node: Node) => void;
}

export interface TreeRendererControls {
  destroy: () => void;
  resetZoom: () => void;
  zoomIn: () => void;
  zoomOut: () => void;
}

/**
 * Creates a tree renderer attached to an SVG element.
 * Returns controls for cleanup and zoom manipulation.
 */
export function createTreeRenderer(
  svg: SVGSVGElement,
  options: TreeRendererOptions
): TreeRendererControls {
  const { tree, expandedPath, highlightedPath, selectedNodeId, onNodeClick } = options;

  // Clear any existing content
  const svgSelection = d3.select(svg);
  svgSelection.selectAll('*').remove();

  // Build hierarchy with collapse logic (uses expandedPath)
  const root = buildHierarchy(tree, expandedPath, true);
  const hierarchy = d3.hierarchy(root);

  // Compute layout using nodeSize for consistent spacing
  const treeLayout = d3.tree<TreeNode>().nodeSize([V_SPACING, H_SPACING]);
  const layoutRoot = treeLayout(hierarchy);

  // Calculate bounds for viewBox
  const bounds = computeBounds(layoutRoot);

  // Setup SVG with zoom behavior
  const { contentG, zoom } = setupSvg(svgSelection, bounds);

  // Render links first (below nodes) - uses highlightedPath for styling
  renderLinks(contentG, layoutRoot, highlightedPath);

  // Render nodes - uses highlightedPath for styling
  renderNodes(contentG, layoutRoot, highlightedPath, selectedNodeId, tree, onNodeClick);

  return {
    destroy: () => {
      svgSelection.selectAll('*').remove();
    },
    resetZoom: () => {
      svgSelection.transition().duration(300).call(zoom.transform, d3.zoomIdentity);
    },
    zoomIn: () => {
      svgSelection.transition().duration(200).call(zoom.scaleBy, 1.5);
    },
    zoomOut: () => {
      svgSelection.transition().duration(200).call(zoom.scaleBy, 0.67);
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
 * Setup SVG with proper viewBox, zoom behavior, and nested group structure.
 */
function setupSvg(
  svg: d3.Selection<SVGSVGElement, unknown, null, undefined>,
  bounds: { minX: number; maxX: number; minY: number; maxY: number }
): {
  contentG: d3.Selection<SVGGElement, unknown, null, undefined>;
  zoom: d3.ZoomBehavior<SVGSVGElement, unknown>;
} {
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

  // Create nested group structure: zoomG receives transforms, contentG holds nodes/links
  const zoomG = svg.append('g').attr('class', 'zoom-container');
  const contentG = zoomG.append('g').attr('class', 'content');

  // Define bounds for constraining pan/zoom to content edges
  const worldBounds: [[number, number], [number, number]] = [
    [viewBoxX, viewBoxY],
    [viewBoxX + width, viewBoxY + height],
  ];

  // Create zoom behavior
  const zoom = d3
    .zoom<SVGSVGElement, unknown>()
    .extent(worldBounds)
    .scaleExtent([1, 4]) // No zoom out past initial view, up to 4x zoom in
    .translateExtent(worldBounds)
    // Filter: allow zoom on wheel events, but only start drag-pan from empty space (not nodes)
    .filter((event: Event) => {
      // Always allow wheel events (zoom)
      if (event.type === 'wheel') return true;
      // For mouse/touch events, only allow if not starting on a node
      const target = event.target as Element | null;
      return !target?.closest('.node');
    })
    .on('zoom', (event: d3.D3ZoomEvent<SVGSVGElement, unknown>) => {
      zoomG.attr('transform', event.transform.toString());
    });

  // Attach zoom to SVG (receives wheel/drag events)
  svg.call(zoom);

  return { contentG, zoom };
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
    const nodeHeight = data.isRoot && data.goal ? ROOT_NODE_HEIGHT : NODE_HEIGHT;

    // Determine fill and stroke colors
    // Selected node: blue border, blue fill
    // Ancestor nodes (on path but not selected): blue fill, light border
    // Other nodes: default gray
    let fill = '#f8fafc';
    let stroke = '#e2e8f0';
    let strokeWidth = 1;

    if (isSelected) {
      fill = '#dbeafe';
      stroke = '#3b82f6';
      strokeWidth = 2;
    } else if (isOnPath) {
      fill = '#dbeafe';
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
      .attr('stroke-width', strokeWidth);

    // Selection glow (only for selected node)
    if (isSelected) {
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

    // Layout: Right column for badges, left side for title
    // Badge column width (fixed)
    const badgeColWidth = 50;
    const badgeColX = NODE_WIDTH - badgeColWidth - 6;
    const contentPadding = 10;

    // Status badge (top-right)
    const status = getStatus(data);
    const statusLabel = getStatusLabel(data);
    const colors = STATUS_COLORS[status];
    const badgeWidth = statusLabel.length * 6 + 12;
    const badgeY = -nodeHeight / 2 + 10;

    nodeG
      .append('rect')
      .attr('class', 'status-badge-bg')
      .attr('x', badgeColX + (badgeColWidth - badgeWidth) / 2)
      .attr('y', badgeY)
      .attr('width', badgeWidth)
      .attr('height', 18)
      .attr('rx', 9)
      .attr('fill', colors.bg);

    nodeG
      .append('text')
      .attr('class', 'status-badge-text')
      .attr('x', badgeColX + badgeColWidth / 2)
      .attr('y', badgeY + 9)
      .attr('dy', '0.35em')
      .attr('text-anchor', 'middle')
      .attr('font-size', '11px')
      .attr('font-weight', 500)
      .attr('fill', colors.text)
      .text(statusLabel);

    // Attempts text (below status badge)
    nodeG
      .append('text')
      .attr('class', 'node-attempts')
      .attr('x', badgeColX + badgeColWidth / 2)
      .attr('y', badgeY + 32)
      .attr('dy', '0.35em')
      .attr('text-anchor', 'middle')
      .attr('font-size', '10px')
      .attr('fill', '#94a3b8')
      .text(`${data.attempts}/${data.maxAttempts}`);

    // Collapsed badge (+N) - floating circle in top-right corner
    if (data.isCollapsed && data.collapsedCount > 0) {
      const badgeRadius = 12;
      const badgeCenterX = NODE_WIDTH - 4;
      const badgeCenterY = -nodeHeight / 2 - 4;

      nodeG
        .append('circle')
        .attr('class', 'collapse-badge-bg')
        .attr('cx', badgeCenterX)
        .attr('cy', badgeCenterY)
        .attr('r', badgeRadius)
        .attr('fill', '#3b82f6');

      nodeG
        .append('text')
        .attr('class', 'collapse-badge-text')
        .attr('x', badgeCenterX)
        .attr('y', badgeCenterY)
        .attr('dy', '0.35em')
        .attr('text-anchor', 'middle')
        .attr('font-size', '10px')
        .attr('font-weight', 600)
        .attr('fill', '#ffffff')
        .text(`+${data.collapsedCount}`);
    }

    // Title text (left side, using foreignObject for wrapping)
    const titleWidth = NODE_WIDTH - badgeColWidth - contentPadding - 12;
    const titleHeight = data.isRoot && data.goal ? nodeHeight - 28 : nodeHeight - 16;
    const titleY = -nodeHeight / 2 + 8;

    nodeG
      .append('foreignObject')
      .attr('class', 'node-title-container')
      .attr('x', contentPadding)
      .attr('y', titleY)
      .attr('width', titleWidth)
      .attr('height', titleHeight)
      .append('xhtml:div')
      .attr('title', data.title)
      .style('font-size', '13px')
      .style('font-weight', '500')
      .style('color', '#1e293b')
      .style('line-height', '1.4')
      .style('overflow', 'hidden')
      .style('display', '-webkit-box')
      .style('-webkit-line-clamp', '2')
      .style('-webkit-box-orient', 'vertical')
      .style('word-wrap', 'break-word')
      .text(data.title);

    // Goal text for root node (bottom left)
    if (data.isRoot && data.goal) {
      nodeG
        .append('foreignObject')
        .attr('class', 'node-goal-container')
        .attr('x', contentPadding)
        .attr('y', nodeHeight / 2 - 20)
        .attr('width', titleWidth)
        .attr('height', 18)
        .append('xhtml:div')
        .style('font-size', '11px')
        .style('color', '#64748b')
        .style('overflow', 'hidden')
        .style('text-overflow', 'ellipsis')
        .style('white-space', 'nowrap')
        .text(data.goal);
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
