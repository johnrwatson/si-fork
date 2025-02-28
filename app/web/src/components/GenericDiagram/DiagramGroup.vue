<template>
  <v-group
    ref="groupRef"
    :config="{
      id: group.uniqueKey,
      x: position.x,
      y: position.y,
      ...(isDeleted && { opacity: 0.5 }),
    }"
    @mouseover="onMouseOver"
    @mouseout="onMouseOut"
  >
    <!-- selection box outline -->
    <v-rect
      v-if="isHovered || isSelected"
      :config="{
        width: nodeWidth + 8,
        height: nodeHeight + 8,
        x: -halfWidth - 4,
        y: -4 - nodeHeaderHeight - GROUP_HEADER_BOTTOM_MARGIN,
        cornerRadius: CORNER_RADIUS + 3,
        stroke: SELECTION_COLOR,
        strokeWidth: isSelected ? 5 : 2,
        listening: false,
      }"
    />
    <!-- box background - also used by layout manager to figure out nodes location and size -->
    <!-- <v-rect
      :config="{
        id: `${group.uniqueKey}--bg`,
        width: nodeWidth,
        height: nodeHeight,
        x: -halfWidth,
        y: 0,
      }"
    /> -->

    <!--  Node Body  -->
    <v-rect
      :config="{
        id: `${group.uniqueKey}--bg`,
        width: nodeWidth,
        height: nodeBodyHeight,
        x: -halfWidth,
        y: 0,
        cornerRadius: CORNER_RADIUS,
        fill: colors.bodyBg,
        fillAfterStrokeEnabled: true,
        stroke: colors.headerBg,
        strokeWidth: 3,
        hitStrokeWidth: 0,
        dash: [8, 8],
        shadowColor: 'black',
        shadowBlur: 8,
        shadowOffset: { x: 3, y: 3 },
        shadowOpacity: 0.4,
        shadowEnabled: false,
      }"
    />

    <!-- resize handles -->
    <!--  left side handle  -->
    <v-line
      :config="{
        points: [-nodeWidth / 2, 0, -nodeWidth / 2, nodeBodyHeight],
        hitStrokeWidth: GROUP_RESIZE_HANDLE_SIZE,
      }"
      @mouseover="onResizeHover('left', $event)"
      @mouseout="onMouseOut"
    />
    <!-- right side handle   -->
    <v-line
      :config="{
        points: [nodeWidth / 2, 0, nodeWidth / 2, nodeBodyHeight],
        hitStrokeWidth: GROUP_RESIZE_HANDLE_SIZE,
      }"
      @mouseover="onResizeHover('right', $event)"
      @mouseout="onMouseOut"
    />
    <!-- Bottom Handle -->
    <v-line
      :config="{
        points: [-nodeWidth / 2, nodeBodyHeight, nodeWidth / 2, nodeBodyHeight],
        hitStrokeWidth: GROUP_RESIZE_HANDLE_SIZE,
      }"
      @mouseover="onResizeHover('bottom', $event)"
      @mouseout="onMouseOut"
    />
    <!-- Bottom Left Handle -->
    <v-circle
      :config="{
        width: GROUP_RESIZE_HANDLE_SIZE,
        height: GROUP_RESIZE_HANDLE_SIZE,
        x: -nodeWidth / 2,
        y: nodeBodyHeight,
      }"
      @mouseover="onResizeHover('bottom-left', $event)"
      @mouseout="onMouseOut"
    />
    <!-- Bottom Right Handle -->
    <v-circle
      :config="{
        width: GROUP_RESIZE_HANDLE_SIZE,
        height: GROUP_RESIZE_HANDLE_SIZE,
        x: nodeWidth / 2,
        y: nodeBodyHeight,
      }"
      @mouseover="onResizeHover('bottom-right', $event)"
      @mouseout="onMouseOut"
    />

    <!-- sockets -->
    <v-group
      :config="{
        x: -halfWidth - 1,
        y: nodeHeaderHeight + SOCKET_MARGIN_TOP,
      }"
    >
      <DiagramNodeSocket
        v-for="(socket, i) in leftSockets"
        :key="socket.uniqueKey"
        :socket="socket"
        :y="i * SOCKET_GAP"
        :connectedEdges="connectedEdgesBySocketKey[socket.uniqueKey]"
        :drawEdgeState="drawEdgeState"
        :nodeWidth="nodeWidth"
        @hover:start="onSocketHoverStart(socket)"
        @hover:end="onSocketHoverEnd(socket)"
      />
    </v-group>

    <v-group
      :config="{
        x: halfWidth + 1,
        y:
          nodeHeaderHeight +
          SOCKET_MARGIN_TOP +
          SOCKET_GAP * leftSockets.length,
      }"
    >
      <DiagramNodeSocket
        v-for="(socket, i) in rightSockets"
        :key="socket.uniqueKey"
        :socket="socket"
        :y="i * SOCKET_GAP"
        :connectedEdges="connectedEdgesBySocketKey[socket.uniqueKey]"
        :drawEdgeState="drawEdgeState"
        :nodeWidth="nodeWidth"
        @hover:start="onSocketHoverStart(socket)"
        @hover:end="onSocketHoverEnd(socket)"
      />
    </v-group>

    <!-- header -->
    <v-group
      :config="{
        x: -halfWidth,
        y: -nodeHeaderHeight - GROUP_HEADER_BOTTOM_MARGIN,
      }"
    >
      <!-- header background -->
      <!--  TODO check with mark what this width should be   -->
      <v-rect
        :config="{
          cornerRadius: CORNER_RADIUS,
          fill: colors.headerBg,
          x: 0,
          y: 0,
          width: headerWidth,
          height: headerTextHeight,
        }"
      />

      <!-- package/type icon -->
      <DiagramIcon
        v-if="group.def.typeIcon"
        :icon="group.def.typeIcon"
        :color="colors.icon"
        :size="GROUP_HEADER_ICON_SIZE"
        :x="5"
        :y="5"
        origin="top-left"
      />

      <!-- header text -->
      <v-text
        ref="titleTextRef"
        :config="{
          x: 42,
          y: 2,
          verticalAlign: 'top',
          align: 'left',
          width: headerWidth - GROUP_HEADER_ICON_SIZE - 2,
          text: group.def.title,
          padding: 6,
          fill: colors.headerText,
          fontSize: GROUP_TITLE_FONT_SIZE,
          fontStyle: 'bold',
          fontFamily: DIAGRAM_FONT_FAMILY,
          listening: false,
          wrap: 'none',
          ellipsis: true,
        }"
      />

      <!-- subtitle text -->
      <v-text
        ref="titleTextRef"
        :config="{
          x: 42,
          y: 20,
          verticalAlign: 'top',
          align: 'left',
          width: headerWidth - GROUP_HEADER_ICON_SIZE - 2,
          text: `${group.def.subtitle}: ${childCount ?? 0}`,
          padding: 6,
          fill: colors.headerText,
          fontSize: GROUP_TITLE_FONT_SIZE,
          fontStyle: 'italic',
          fontFamily: DIAGRAM_FONT_FAMILY,
          listening: false,
          wrap: 'none',
          ellipsis: true,
        }"
      />
      />
    </v-group>

    <!-- status icons -->
    <v-group
      v-if="group.def.statusIcons?.length"
      :config="{
        x: halfWidth - group.def.statusIcons.length * 22 - 2,
        y: 0,
      }"
    >
      <DiagramIcon
        v-for="(statusIcon, i) in group.def.statusIcons"
        :key="`status-icon-${i}`"
        :icon="statusIcon.icon"
        :color="statusIcon.color || diagramConfig?.toneColors?.[statusIcon.tone!] || diagramConfig?.toneColors?.neutral || '#AAA'"
        :size="20"
        :x="i * 22"
        :y="nodeBodyHeight - 5"
        origin="bottom-left"
      />
    </v-group>

    <!--  spinner overlay  -->
    <v-group
      ref="overlay"
      :config="{
        x: -halfWidth,
        y: 0,
        opacity: 0,
        listening: false,
      }"
    >
      <!--  transparent overlay  -->
      <v-rect
        :config="{
          width: nodeWidth,
          height: nodeBodyHeight,
          x: 0,
          y: 0,
          cornerRadius: [0, 0, CORNER_RADIUS, CORNER_RADIUS],
          fill: 'rgba(255,255,255,0.70)',
        }"
      />
      <DiagramIcon
        icon="loader"
        :color="diagramConfig?.toneColors?.['info'] || '#AAA'"
        :size="overlayIconSize"
        :x="halfWidth"
        :y="nodeBodyHeight / 2"
      />
    </v-group>

    <!-- added/modified indicator -->
    <DiagramIcon
      v-if="isAdded || isModified"
      :icon="isAdded ? 'plus' : 'tilde'"
      :bgColor="
        isAdded
          ? diagramConfig?.toneColors?.success
          : diagramConfig?.toneColors?.warning
      "
      circleBg
      :color="theme === 'dark' ? '#000' : '#FFF'"
      :size="GROUP_HEADER_ICON_SIZE"
      :x="halfWidth - GROUP_HEADER_ICON_SIZE / 2"
      :y="
        -nodeHeaderHeight +
        GROUP_HEADER_ICON_SIZE / 2 -
        GROUP_HEADER_BOTTOM_MARGIN +
        (nodeHeaderHeight - GROUP_HEADER_ICON_SIZE) / 2
      "
      origin="center"
    />
  </v-group>
</template>

<script lang="ts" setup>
import { computed, nextTick, PropType, ref, watch } from "vue";
import * as _ from "lodash-es";
import tinycolor from "tinycolor2";

import { KonvaEventObject } from "konva/lib/Node";
import { Vector2d } from "konva/lib/types";
import { useTheme } from "@si/vue-lib/design-system";
import DiagramNodeSocket from "@/components/GenericDiagram/DiagramNodeSocket.vue";
import {
  SOCKET_GAP,
  SOCKET_MARGIN_TOP,
  CORNER_RADIUS,
  DEFAULT_NODE_COLOR,
  DIAGRAM_FONT_FAMILY,
  SELECTION_COLOR,
  GROUP_HEADER_BOTTOM_MARGIN,
  GROUP_TITLE_FONT_SIZE,
  GROUP_RESIZE_HANDLE_SIZE,
  GROUP_HEADER_ICON_SIZE,
} from "@/components/GenericDiagram/diagram_constants";
import { useComponentsStore } from "@/store/components.store";
import {
  DiagramDrawEdgeState,
  DiagramEdgeData,
  DiagramElementUniqueKey,
  DiagramGroupData,
  Size2D,
  SideAndCornerIdentifiers,
  DiagramSocketData,
  ElementHoverMeta,
} from "./diagram_types";
import DiagramIcon from "./DiagramIcon.vue";
import { useDiagramConfig } from "./utils/use-diagram-context-provider";

const props = defineProps({
  group: {
    type: Object as PropType<DiagramGroupData>,
    required: true,
  },
  connectedEdges: {
    type: Object as PropType<DiagramEdgeData[]>,
    default: () => ({}),
  },
  tempPosition: {
    type: Object as PropType<Vector2d>,
  },
  tempSize: {
    type: Object as PropType<Size2D>,
  },
  drawEdgeState: {
    type: Object as PropType<DiagramDrawEdgeState>,
    default: () => ({}),
  },
  isHovered: Boolean,
  isSelected: Boolean,
});

const emit = defineEmits<{
  (e: "hover:start", meta?: ElementHoverMeta): void;
  (e: "hover:end"): void;
  (e: "resize"): void;
}>();

const { theme } = useTheme();
const diagramConfig = useDiagramConfig();

const titleTextRef = ref();
const groupRef = ref();

const size = computed(
  () => props.tempSize || props.group.def.size || { width: 500, height: 500 },
);

const isDeleted = computed(() => props.group.def.changeStatus === "deleted");
const isModified = computed(() => props.group.def.changeStatus === "modified");
const isAdded = computed(() => props.group.def.changeStatus === "added");

const childCount = computed(() => {
  const mappedChildren = _.map(
    props.group.def.childNodeIds,
    (child) => useComponentsStore().componentsByNodeId[child],
  );

  const undeletedChildren = _.filter(mappedChildren, (child) =>
    _.isNil(child?.deletedInfo),
  );

  return undeletedChildren.length;
});

const overlayIconSize = computed(() => nodeWidth.value / 3);

const nodeWidth = computed(() => size.value.width);
const halfWidth = computed(() => nodeWidth.value / 2);
const headerWidth = computed(() =>
  !props.group.def.changeStatus || props.group.def.changeStatus === "unmodified"
    ? nodeWidth.value
    : nodeWidth.value - GROUP_HEADER_ICON_SIZE - 4,
);

const actualSockets = computed(() =>
  _.filter(
    props.group.sockets,
    (s) =>
      s.def.label !== "Frame" && s.parent.def.nodeType !== "configurationFrame",
  ),
);

const leftSockets = computed(() =>
  _.filter(actualSockets.value, (s) => s.def.nodeSide === "left"),
);
const rightSockets = computed(() =>
  _.filter(actualSockets.value, (s) => s.def.nodeSide === "right"),
);
const connectedEdgesBySocketKey = computed(() => {
  const lookup: Record<DiagramElementUniqueKey, DiagramEdgeData[]> = {};
  _.each(props.connectedEdges, (edge) => {
    lookup[edge.fromSocketKey] ||= [];
    lookup[edge.fromSocketKey]!.push(edge); // eslint-disable-line @typescript-eslint/no-non-null-assertion
    lookup[edge.toSocketKey] ||= [];
    lookup[edge.toSocketKey]!.push(edge); // eslint-disable-line @typescript-eslint/no-non-null-assertion
  });
  return lookup;
});

const headerTextHeight = ref(20);
watch(
  [nodeWidth, () => props.group.def.title, () => props.group.def.subtitle],
  () => {
    // we have to let the new header be drawn on the canvas before we can check the height
    nextTick(recalcHeaderHeight);
  },
  { immediate: true },
);

function recalcHeaderHeight() {
  headerTextHeight.value =
    titleTextRef.value?.getNode()?.getSelfRect().height || 20;
  headerTextHeight.value *= 1.7;
}

const nodeHeaderHeight = computed(() => headerTextHeight.value);
const nodeBodyHeight = computed(() => size.value.height);
const nodeHeight = computed(
  () =>
    nodeHeaderHeight.value + GROUP_HEADER_BOTTOM_MARGIN + nodeBodyHeight.value,
);

const position = computed(() => props.tempPosition || props.group.def.position);

watch([nodeWidth, nodeHeight, position], () => {
  // we call on nextTick to let the component actually update itself on the stage first
  // because parent responds to this event by finding shapes on the stage and looking at location/dimensions
  nextTick(() => emit("resize"));
});

const colors = computed(() => {
  const primaryColor = tinycolor(props.group.def.color || DEFAULT_NODE_COLOR);

  // body bg
  const bodyBgHsl = primaryColor.toHsl();
  bodyBgHsl.l = theme.value === "dark" ? 0.08 : 0.95;
  const bodyBg = tinycolor(bodyBgHsl);

  const bodyText = theme.value === "dark" ? "#FFF" : "#000";
  let headerText;
  if (primaryColor.toHsl().l < 0.5) {
    headerText = "#FFF";
  } else {
    headerText = "#000";
  }
  return {
    headerBg: primaryColor.toRgbString(),
    icon: "#000",
    headerText,
    bodyBg: bodyBg.toRgbString(),
    bodyText,
  };
});

function onMouseOver(evt: KonvaEventObject<MouseEvent>) {
  evt.cancelBubble = true;
  emit("hover:start");
}

function onResizeHover(
  direction: SideAndCornerIdentifiers,
  evt: KonvaEventObject<MouseEvent>,
) {
  evt.cancelBubble = true;
  emit("hover:start", { type: "resize", direction });
}

function onSocketHoverStart(socket: DiagramSocketData) {
  emit("hover:start", { type: "socket", socket });
}

function onSocketHoverEnd(_socket: DiagramSocketData) {
  emit("hover:end");
}

function onMouseOut(_e: KonvaEventObject<MouseEvent>) {
  emit("hover:end");
}
</script>
