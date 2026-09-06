#!/usr/bin/env python3
"""
X-Men Legends II XMLB Editor - PyQt5 Desktop Application
Visual editor for Athanor engine XMLB UI binary format.
"""

import sys
import os
import struct
from pathlib import Path

from PyQt5.QtWidgets import (
    QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout,
    QTreeWidget, QTreeWidgetItem, QSplitter, QFileDialog,
    QMessageBox, QInputDialog, QMenu, QAction, QLabel,
    QScrollArea, QFrame, QPushButton, QToolBar, QStatusBar,
    QLineEdit, QSpinBox, QDoubleSpinBox, QComboBox, QGridLayout,
    QGroupBox, QCheckBox, QDialog, QDialogButtonBox, QFormLayout,
    QTabWidget, QTextEdit, QGraphicsView, QGraphicsScene,
    QGraphicsRectItem, QGraphicsTextItem, QGraphicsItem,
    QStyleOptionGraphicsItem, QSizePolicy
)
from PyQt5.QtCore import Qt, QPointF, QRectF, pyqtSignal, QSize
from PyQt5.QtGui import QColor, QPen, QBrush, QFont, QPainter, QIcon

# ============================================================
# XMLB Format Parser (same as xmlb_core.py)
# ============================================================

MAGIC = 0x000011B1
HEADER_SIZE = 24
NODE_SIZE = 32


class XMLBNode:
    """Represents a single widget node in an XMLB file."""
    def __init__(self, index, file_offset, type_str, name_str, props):
        self.index = index
        self.file_offset = file_offset
        self.type = type_str
        self.name = name_str
        self.props = props  # List of (key, value, is_raw) tuples

    def __repr__(self):
        return f"Node({self.index}: {self.type} '{self.name}')"

    def get_prop(self, key):
        for k, v, _ in self.props:
            if k == key:
                return v
        return None


class XMLBFile:
    """Represents a parsed XMLB file."""

    def __init__(self):
        self.data = bytearray()
        self.path = ""
        self.magic = 0
        self.version = 0
        self.strtab_off = 0
        self.unk1 = 0
        self.str_count = 0
        self.unk2 = 0
        self.strings = {}
        self.strings_rev = {}
        self.nodes = []
        self.modified = False

    @classmethod
    def load(cls, path):
        """Load and parse an XMLB file."""
        with open(path, 'rb') as f:
            data = bytearray(f.read())

        xmlb = cls()
        xmlb.path = path
        xmlb.data = data

        # Parse header
        xmlb.magic = struct.unpack_from('<I', data, 0)[0]
        xmlb.version = struct.unpack_from('<I', data, 4)[0]
        xmlb.strtab_off = struct.unpack_from('<I', data, 8)[0]
        xmlb.unk1 = struct.unpack_from('<I', data, 12)[0]
        xmlb.str_count = struct.unpack_from('<I', data, 16)[0]
        xmlb.unk2 = struct.unpack_from('<I', data, 20)[0]

        if xmlb.magic != MAGIC:
            raise ValueError(f"Invalid magic: 0x{xmlb.magic:08X} (expected 0x{MAGIC:08X})")

        xmlb._build_string_table()
        xmlb._parse_nodes()
        return xmlb

    def _build_string_table(self):
        pos = self.strtab_off
        for _ in range(self.str_count):
            if pos >= len(self.data):
                break
            if self.data[pos] == 0:
                pos += 1
                continue
            end = pos
            while end < len(self.data) and self.data[end] != 0:
                end += 1
            s = bytes(self.data[pos:end]).decode('latin-1', errors='replace')
            self.strings[pos] = s
            self.strings_rev[s] = pos
            pos = end + 1

    def _parse_nodes(self):
        for n in range(self.str_count):
            base = 0x18 + n * NODE_SIZE
            if base + NODE_SIZE > len(self.data):
                break

            offsets = []
            for i in range(8):
                off = struct.unpack_from('<I', self.data, base + i * 4)[0]
                offsets.append(off)

            type_off = offsets[0]
            name_off = offsets[1]

            type_str = self.strings.get(type_off)
            if type_off == 0xFFFFFFFF:
                type_str = None
            elif type_str is None:
                type_str = f"<0x{type_off:04X}>"

            name_str = self.strings.get(name_off)
            if name_off == 0xFFFFFFFF:
                name_str = None
            elif name_str is None:
                name_str = f"<0x{name_off:04X}>"

            props = []
            for i in range(3):
                key_off = offsets[2 + i * 2]
                val = offsets[3 + i * 2]

                if key_off == 0xFFFFFFFF:
                    continue

                key = self.strings.get(key_off)
                if key is None:
                    key = f"<0x{key_off:04X}>"

                if val == 0xFFFFFFFF:
                    val_str = None
                    val_raw = None
                elif val in self.strings:
                    val_str = self.strings[val]
                    val_raw = None
                else:
                    val_str = str(val)
                    val_raw = val

                props.append((key, val_str, val_raw))

            node = XMLBNode(n, base, type_str, name_str, props)
            self.nodes.append(node)

    def get_string_offset(self, s):
        return self.strings_rev.get(s)

    def add_string(self, s):
        if s in self.strings_rev:
            return self.strings_rev[s]
        end = self.strtab_off
        while end < len(self.data) and self.data[end] != 0:
            end += 1
        while end < len(self.data) and self.data[end] == 0:
            end += 1
        new_off = end
        self.data.extend(s.encode('latin-1'))
        self.data.append(0)
        self.strings[new_off] = s
        self.strings_rev[s] = new_off
        self.str_count += 1
        struct.pack_into('<I', self.data, 16, self.str_count)
        self.modified = True
        return new_off

    def set_property(self, node_idx, prop_idx, value):
        if node_idx >= len(self.nodes):
            return
        if prop_idx >= len(self.nodes[node_idx].props):
            return

        key, old_val_str, old_val_raw = self.nodes[node_idx].props[prop_idx]

        if isinstance(value, str):
            off = self.get_string_offset(value)
            if off is None:
                off = self.add_string(value)
            new_val = off
            new_val_str = value
            new_val_raw = None
        else:
            new_val = int(value)
            new_val_str = str(new_val)
            new_val_raw = new_val

        self.nodes[node_idx].props[prop_idx] = (key, new_val_str, new_val_raw)
        prop_off = self.nodes[node_idx].file_offset + 16 + prop_idx * 8 + 4
        struct.pack_into('<I', self.data, prop_off, new_val)
        self.modified = True

    def save(self, path=None):
        if path is None:
            path = self.path
        with open(path, 'wb') as f:
            f.write(self.data)
        self.path = path
        self.modified = False


def load_xmlb(path):
    return XMLBFile.load(path)


# ============================================================
# Graphics Items for Preview Canvas
# ============================================================

NODE_COLORS = {
    'MENU': QColor('#569cd6'),
    'MENU_ITEM_MODEL': QColor('#4ec9b0'),
    'animtext': QColor('#dcdcaa'),
    'animtext_scene': QColor('#c586c0'),
    'style': QColor('#ce9178'),
    'text': QColor('#b5cea8'),
    'alpha': QColor('#e5c07b'),
    'time': QColor('#98c379'),
    'name': QColor('#d7ba7d'),
    'type': QColor('#569cd6'),
    'item': QColor('#4ec9b0'),
    'model': QColor('#c586c0'),
    'mark': QColor('#f1fa8c'),
}

DEFAULT_COLOR = QColor('#858585')
SELECTED_COLOR = QColor('#f1fa8c')


class NodeGraphicsItem(QGraphicsRectItem):
    """Visual representation of an XMLB node on the canvas."""

    def __init__(self, node, parent=None):
        super().__init__(parent)
        self.node = node
        self.setRect(0, 0, 100, 30)
        self.setFlags(
            QGraphicsItem.ItemIsSelectable |
            QGraphicsItem.ItemIsMovable |
            QGraphicsItem.ItemSendsGeometryChanges
        )
        self.setAcceptHoverEvents(True)
        self._selected = False
        self.update_appearance()

    def update_appearance(self):
        color = NODE_COLORS.get(self.node.type, DEFAULT_COLOR)
        if self.isSelected():
            color = SELECTED_COLOR
        self.setBrush(QBrush(color.lighter(150)))
        self.setPen(QPen(color, 2 if self.isSelected() else 1))

    def paint(self, painter, option, widget=None):
        self.update_appearance()
        super().paint(painter, option, widget)

        # Draw node info
        painter.setPen(Qt.black)
        font = QFont('Consolas', 8)
        painter.setFont(font)
        text = f"{self.node.type}\n{self.node.name or ''}"
        painter.drawText(self.rect().adjusted(4, 2, -4, -2), Qt.AlignCenter, text)

    def itemChange(self, change, value):
        if change == QGraphicsItem.ItemPositionHasChanged:
            # Update the node's properties when moved
            self.update_node_position()
        return super().itemChange(change, value)

    def update_node_position(self):
        pos = self.pos()
        # Find 'mark' and 'item' properties and update them
        for i, (key, val, is_raw) in enumerate(self.node.props):
            if key == 'mark' and is_raw:
                # mark is vertical position index
                pass  # We'll implement coordinate mapping later
            if key == 'item' and is_raw:
                pass


class PreviewCanvas(QGraphicsView):
    """Canvas for previewing and editing XMLB layout."""

    node_selected = pyqtSignal(object)  # XMLBNode

    def __init__(self, parent=None):
        super().__init__(parent)
        self.scene = QGraphicsScene(self)
        self.setScene(self.scene)
        self.setRenderHint(QPainter.Antialiasing)
        self.setDragMode(QGraphicsView.RubberBandDrag)
        self.setTransformationAnchor(QGraphicsView.AnchorUnderMouse)
        self.setResizeAnchor(QGraphicsView.AnchorUnderMouse)

        # Grid settings
        self.grid_size = 20
        self.show_grid = True

        # Aspect ratio
        self.aspect_ratio = '4:3'
        self._setup_canvas()

    def _setup_canvas(self):
        if self.aspect_ratio == '4:3':
            w, h = 1024, 768
        else:
            w, h = 1280, 720
        self.scene.setSceneRect(0, 0, w, h)

    def set_aspect_ratio(self, ratio):
        self.aspect_ratio = ratio
        self._setup_canvas()
        self.fitInView(self.scene.sceneRect(), Qt.KeepAspectRatio)

    def load_xmlb(self, xmlb_file):
        self.scene.clear()
        self.node_items = {}

        # Draw background grid
        if self.show_grid:
            pen = QPen(QColor('#3e3e3e'), 1)
            for x in range(0, int(self.scene.width()), self.grid_size):
                self.scene.addLine(x, 0, x, self.scene.height(), pen)
            for y in range(0, int(self.scene.height()), self.grid_size):
                self.scene.addLine(0, y, self.scene.width(), y, pen)

        # Create visual items for each node
        for node in xmlb_file.nodes:
            item = self.create_node_item(node)
            if item:
                self.scene.addItem(item)
                self.node_items[node.index] = item

        self.fitInView(self.scene.sceneRect(), Qt.KeepAspectRatio)

    def create_node_item(self, node):
        """Create a graphics item for a node based on its type and properties."""
        # Calculate position from properties
        x, y = 50, 50
        width, height = 200, 32

        for key, val, is_raw in node.props:
            if key == 'mark' and is_raw:
                # mark is an index, multiply by spacing
                y = 50 + val * 40
            elif key == 'item' and is_raw:
                # item is absolute Y coordinate
                if val > 100:
                    y = val
                else:
                    y = 50 + val * 40

        # Adjust size by type
        if node.type == 'MENU_ITEM_MODEL':
            width, height = 300, 40
        elif node.type == 'animtext':
            width, height = 250, 24
        elif node.type == 'style':
            width, height = 100, 100

        item = NodeGraphicsItem(node)
        item.setRect(0, 0, width, height)
        item.setPos(x, y)
        return item


# ============================================================
# Property Editor Widget
# ============================================================

class PropertyEditor(QWidget):
    """Widget for editing node properties."""

    property_changed = pyqtSignal(int, int, object)  # node_idx, prop_idx, value

    def __init__(self, parent=None):
        super().__init__(parent)
        self.current_node = None
        self.current_node_idx = -1
        self.prop_widgets = []  # List of (prop_idx, widget)

        layout = QVBoxLayout(self)
        layout.setContentsMargins(8, 8, 8, 8)
        layout.setSpacing(6)

        self.info_label = QLabel("No node selected")
        self.info_label.setWordWrap(True)
        self.info_label.setStyleSheet("color: #858585; font-size: 11px;")
        layout.addWidget(self.info_label)

        self.scroll = QScrollArea()
        self.scroll.setWidgetResizable(True)
        self.scroll.setHorizontalScrollBarPolicy(Qt.ScrollBarAlwaysOff)
        self.container = QWidget()
        self.form_layout = QFormLayout(self.container)
        self.form_layout.setFieldGrowthPolicy(QFormLayout.ExpandingFieldsGrow)
        self.scroll.setWidget(self.container)
        layout.addWidget(self.scroll)

    def set_node(self, node, node_idx):
        self.current_node = node
        self.current_node_idx = node_idx

        # Clear existing widgets
        for _, widget in self.prop_widgets:
            widget.deleteLater()
        self.prop_widgets.clear()

        if node is None:
            self.info_label.setText("No node selected")
            return

        self.info_label.setText(
            f"<b>Node {node.index}</b><br>"
            f"Type: <span style='color:#569cd6'>{node.type}</span><br>"
            f"Name: <span style='color:#d4d4d4'>{node.name or '???'}</span><br>"
            f"Offset: 0x{node.file_offset:04X}"
        )

        for i, (key, val, is_raw) in enumerate(node.props):
            label = QLabel(key)
            label.setStyleSheet("color: #9cdcfe; font-family: Consolas;")

            if is_raw:
                widget = QSpinBox()
                widget.setRange(-10000, 10000)
                widget.setValue(int(val) if val else 0)
                widget.setStyleSheet("""
                    QSpinBox { background: #3c3c3c; color: #d4d4d4; border: 1px solid #3e3e3e; padding: 4px; }
                """)
                widget.valueChanged.connect(
                    lambda v, idx=i: self.property_changed.emit(self.current_node_idx, idx, v)
                )
            else:
                widget = QLineEdit()
                widget.setText(val if val else "")
                widget.setStyleSheet("""
                    QLineEdit { background: #3c3c3c; color: #d4d4d4; border: 1px solid #3e3e3e; padding: 4px; }
                """)
                widget.textChanged.connect(
                    lambda t, idx=i: self.property_changed.emit(self.current_node_idx, idx, t)
                )

            self.form_layout.addRow(label, widget)
            self.prop_widgets.append((i, widget))

    def clear(self):
        self.set_node(None, -1)


# ============================================================
# Node Tree Widget
# ============================================================

class NodeTreeWidget(QTreeWidget):
    """Tree view of XMLB nodes."""

    node_selected = pyqtSignal(int)  # node index

    def __init__(self, parent=None):
        super().__init__(parent)
        self.setHeaderLabels(["Index", "Type", "Name"])
        self.setColumnWidth(0, 50)
        self.setColumnWidth(1, 150)
        self.setAlternatingRowColors(True)
        self.setRootIsDecorated(False)
        self.itemSelectionChanged.connect(self._on_selection_changed)

    def load_nodes(self, nodes):
        self.clear()
        self.nodes = nodes
        for node in nodes:
            item = QTreeWidgetItem([
                str(node.index),
                node.type or "???",
                node.name or "???"
            ])
            item.setData(0, Qt.UserRole, node.index)
            # Color by type
            color = NODE_COLORS.get(node.type, DEFAULT_COLOR)
            item.setForeground(1, color)
            self.addTopLevelItem(item)

    def _on_selection_changed(self):
        items = self.selectedItems()
        if items:
            idx = items[0].data(0, Qt.UserRole)
            self.node_selected.emit(idx)

    def select_node(self, idx):
        for i in range(self.topLevelItemCount()):
            item = self.topLevelItem(i)
            if item.data(0, Qt.UserRole) == idx:
                self.setCurrentItem(item)
                break


# ============================================================
# Main Window
# ============================================================

class MainWindow(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("X-Men Legends II XMLB Editor")
        self.resize(1400, 900)

        self.xmlb_file = None
        self.current_file_path = ""

        self._setup_ui()
        self._setup_actions()

    def _setup_ui(self):
        # Central widget with splitter
        central = QWidget()
        self.setCentralWidget(central)
        main_layout = QHBoxLayout(central)
        main_layout.setContentsMargins(0, 0, 0, 0)

        # Left: Node Tree
        left_panel = QFrame()
        left_panel.setFrameShape(QFrame.StyledPanel)
        left_panel.setMaximumWidth(350)
        left_panel.setMinimumWidth(280)
        left_layout = QVBoxLayout(left_panel)

        tree_header = QHBoxLayout()
        tree_header.addWidget(QLabel("Node Tree"))
        self.node_count_label = QLabel("0")
        self.node_count_label.setStyleSheet("color: #858585;")
        tree_header.addWidget(self.node_count_label)
        tree_header.addStretch()
        left_layout.addLayout(tree_header)

        self.tree = NodeTreeWidget()
        self.tree.node_selected.connect(self._on_node_selected)
        left_layout.addWidget(self.tree)

        # Center: Preview Canvas
        center_panel = QFrame()
        center_panel.setFrameShape(QFrame.StyledPanel)
        center_layout = QVBoxLayout(center_panel)
        center_layout.setContentsMargins(0, 0, 0, 0)

# Canvas toolbar
        canvas_toolbar = QToolBar()
        canvas_toolbar.setIconSize(QSize(16, 16))
        self.aspect_btn = QPushButton("4:3")
        self.aspect_btn.setCheckable(True)
        self.aspect_btn.setChecked(True)
        self.aspect_btn.clicked.connect(self._toggle_aspect)
        canvas_toolbar.addWidget(self.aspect_btn)
        canvas_toolbar.addSeparator()
        zoom_in = QPushButton("+")
        zoom_in.clicked.connect(lambda: self.canvas.scale(1.2, 1.2))
        canvas_toolbar.addWidget(zoom_in)
        zoom_out = QPushButton("-")
        zoom_out.clicked.connect(lambda: self.canvas.scale(0.8, 0.8))
        canvas_toolbar.addWidget(zoom_out)
        reset = QPushButton("Reset")
        reset.clicked.connect(self._reset_view)
        canvas_toolbar.addWidget(reset)
        center_layout.addWidget(canvas_toolbar)

        self.canvas = PreviewCanvas()
        self.canvas.node_selected.connect(self._on_canvas_node_selected)
        center_layout.addWidget(self.canvas)

        canvas_toolbar = QToolBar()
        canvas_toolbar.setIconSize(QSize(16, 16))
        self.aspect_btn = QPushButton("4:3")
        self.aspect_btn.setCheckable(True)
        self.aspect_btn.setChecked(True)
        self.aspect_btn.clicked.connect(self._toggle_aspect)
        canvas_toolbar.addWidget(self.aspect_btn)
        canvas_toolbar.addSeparator()
        zoom_in = QPushButton("+")
        zoom_in.clicked.connect(lambda: self.canvas.scale(1.2, 1.2))
        canvas_toolbar.addWidget(zoom_in)
        zoom_out = QPushButton("-")
        zoom_out.clicked.connect(lambda: self.canvas.scale(0.8, 0.8))
        canvas_toolbar.addWidget(zoom_out)
        reset = QPushButton("Reset")
        reset.clicked.connect(self._reset_view)
        canvas_toolbar.addWidget(reset)
        center_layout.addWidget(canvas_toolbar)

        # Right: Property Editor
        right_panel = QFrame()
        right_panel.setFrameShape(QFrame.StyledPanel)
        right_panel.setMaximumWidth(400)
        right_panel.setMinimumWidth(320)
        right_layout = QVBoxLayout(right_panel)

        right_layout.addWidget(QLabel("Properties"))
        self.prop_editor = PropertyEditor()
        self.prop_editor.property_changed.connect(self._on_property_changed)
        right_layout.addWidget(self.prop_editor)

        # Add to main splitter
        splitter = QSplitter(Qt.Horizontal)
        splitter.addWidget(left_panel)
        splitter.addWidget(center_panel)
        splitter.addWidget(right_panel)
        splitter.setSizes([300, 700, 350])
        main_layout.addWidget(splitter)

        # Status bar
        self.status_bar = QStatusBar()
        self.setStatusBar(self.status_bar)
        self.status_file = QLabel("No file loaded")
        self.status_nodes = QLabel("Nodes: 0")
        self.status_strings = QLabel("Strings: 0")
        self.status_modified = QLabel("")
        self.status_modified.setStyleSheet("color: #f1fa8c; font-weight: bold;")
        self.status_bar.addWidget(self.status_file, 1)
        self.status_bar.addWidget(self.status_nodes)
        self.status_bar.addWidget(self.status_strings)
        self.status_bar.addWidget(self.status_modified)

    def _setup_actions(self):
        # Toolbar
        toolbar = QToolBar("Main Toolbar")
        toolbar.setMovable(False)
        self.addToolBar(toolbar)

        open_action = QAction("Open", self)
        open_action.setShortcut("Ctrl+O")
        open_action.triggered.connect(self.open_file)
        toolbar.addAction(open_action)

        save_action = QAction("Save", self)
        save_action.setShortcut("Ctrl+S")
        save_action.triggered.connect(self.save_file)
        toolbar.addAction(save_action)

        save_as_action = QAction("Save As", self)
        save_as_action.setShortcut("Ctrl+Shift+S")
        save_as_action.triggered.connect(self.save_file_as)
        toolbar.addAction(save_as_action)

        toolbar.addSeparator()

        deploy_action = QAction("Deploy to Game", self)
        deploy_action.triggered.connect(self.deploy_to_game)
        toolbar.addAction(deploy_action)

        toolbar.addSeparator()

        export_action = QAction("Export Text", self)
        export_action.triggered.connect(self.export_text)
        toolbar.addAction(export_action)

        # Menu bar
        menubar = self.menuBar()
        file_menu = menubar.addMenu("File")
        file_menu.addAction(open_action)
        file_menu.addAction(save_action)
        file_menu.addAction(save_as_action)
        file_menu.addSeparator()
        file_menu.addAction(deploy_action)
        file_menu.addSeparator()
        file_menu.addAction(export_action)

        view_menu = menubar.addMenu("View")
        self.aspect_action = QAction("4:3 / 16:9", self)
        self.aspect_action.setCheckable(True)
        self.aspect_action.setChecked(True)
        self.aspect_action.triggered.connect(self._toggle_aspect)
        view_menu.addAction(self.aspect_action)

    def open_file(self):
        path, _ = QFileDialog.getOpenFileName(
            self, "Open XMLB File", "", "XMLB Files (*.xmlb *.XMLB);;All Files (*)"
        )
        if path:
            self.load_file(path)

    def load_file(self, path):
        try:
            self.xmlb_file = load_xmlb(path)
            self.current_file_path = path
            self._update_ui()
            self.status_file.setText(f"File: {Path(path).name}")
            self.status_modified.setText("")
        except Exception as e:
            QMessageBox.critical(self, "Error", f"Failed to load XMLB:\n{str(e)}")

    def _update_ui(self):
        if not self.xmlb_file:
            return

        self.tree.load_nodes(self.xmlb_file.nodes)
        self.canvas.load_xmlb(self.xmlb_file)
        self.prop_editor.clear()

        self.node_count_label.setText(str(len(self.xmlb_file.nodes)))
        self.status_nodes.setText(f"Nodes: {len(self.xmlb_file.nodes)}")
        self.status_strings.setText(f"Strings: {len(self.xmlb_file.strings)}")

    def _on_node_selected(self, idx):
        if not self.xmlb_file or idx >= len(self.xmlb_file.nodes):
            return
        node = self.xmlb_file.nodes[idx]
        self.prop_editor.set_node(node, idx)

        # Also select on canvas
        if hasattr(self.canvas, 'node_items') and idx in self.canvas.node_items:
            self.canvas.node_items[idx].setSelected(True)

    def _on_canvas_node_selected(self, node):
        self.tree.select_node(node.index)
        self.prop_editor.set_node(node, node.index)

    def _on_property_changed(self, node_idx, prop_idx, value):
        if not self.xmlb_file:
            return
        self.xmlb_file.set_property(node_idx, prop_idx, value)
        self.status_modified.setText("Modified")
        # Update canvas if position changed
        self.canvas.load_xmlb(self.xmlb_file)

    def save_file(self):
        if not self.xmlb_file:
            return
        if self.current_file_path:
            self.xmlb_file.save(self.current_file_path)
            self.status_modified.setText("")
            self.status_bar.showMessage(f"Saved: {self.current_file_path}", 3000)
        else:
            self.save_file_as()

    def save_file_as(self):
        if not self.xmlb_file:
            return
        path, _ = QFileDialog.getSaveFileName(
            self, "Save XMLB File", self.current_file_path, "XMLB Files (*.xmlb)"
        )
        if path:
            self.xmlb_file.save(path)
            self.current_file_path = path
            self.status_file.setText(f"File: {Path(path).name}")
            self.status_modified.setText("")

    def deploy_to_game(self):
        if not self.xmlb_file:
            return
        # Default game directory
        game_dir = Path("D:/My Games/X-Men Legends II Rise of Apocalypse/UI/menus")
        if not game_dir.exists():
            game_dir = Path(QFileDialog.getExistingDirectory(self, "Select Game UI Directory"))
            if not game_dir:
                return

        dest = game_dir / Path(self.current_file_path).name
        try:
            self.xmlb_file.save(str(dest))
            self.status_bar.showMessage(f"Deployed to: {dest}", 5000)
        except Exception as e:
            QMessageBox.critical(self, "Deploy Failed", str(e))

    def export_text(self):
        if not self.xmlb_file:
            return
        text = self.xmlb_file.export_text()
        path, _ = QFileDialog.getSaveFileName(
            self, "Export as Text", self.current_file_path + ".txt", "Text Files (*.txt)"
        )
        if path:
            with open(path, 'w', encoding='utf-8') as f:
                f.write(text)
            self.status_bar.showMessage(f"Exported to: {path}", 3000)

    def _toggle_aspect(self):
        if self.aspect_btn.isChecked():
            self.canvas.set_aspect_ratio('4:3')
            self.aspect_btn.setText("4:3")
            self.aspect_action.setChecked(True)
        else:
            self.canvas.set_aspect_ratio('16:9')
            self.aspect_btn.setText("16:9")
            self.aspect_action.setChecked(False)

    def _reset_view(self):
        self.canvas.fitInView(self.canvas.scene.sceneRect(), Qt.KeepAspectRatio)


def main():
    app = QApplication(sys.argv)
    app.setStyle("Fusion")

    # Dark theme
    palette = app.palette()
    palette.setColor(palette.Window, QColor(0x1e, 0x1e, 0x1e))
    palette.setColor(palette.WindowText, QColor(0xd4, 0xd4, 0xd4))
    palette.setColor(palette.Base, QColor(0x25, 0x25, 0x26))
    palette.setColor(palette.AlternateBase, QColor(0x2d, 0x2d, 0x2d))
    palette.setColor(palette.ToolTipBase, QColor(0x2d, 0x2d, 0x2d))
    palette.setColor(palette.ToolTipText, QColor(0xd4, 0xd4, 0xd4))
    palette.setColor(palette.Text, QColor(0xd4, 0xd4, 0xd4))
    palette.setColor(palette.Button, QColor(0x2d, 0x2d, 0x2d))
    palette.setColor(palette.ButtonText, QColor(0xd4, 0xd4, 0xd4))
    palette.setColor(palette.BrightText, Qt.red)
    palette.setColor(palette.Link, QColor(0x56, 0x9c, 0xd6))
    palette.setColor(palette.Highlight, QColor(0x09, 0x47, 0x71))
    palette.setColor(palette.HighlightedText, Qt.white)
    app.setPalette(palette)

    window = MainWindow()

    # Auto-load sample if exists
    sample = Path("D:/re-lab-share/xmen2_mod/xmlb_samples/options.XMLB")
    if sample.exists():
        window.load_file(str(sample))

    window.show()
    sys.exit(app.exec_())


if __name__ == '__main__':
    main()