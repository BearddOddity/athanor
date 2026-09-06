#!/usr/bin/env python3
import sys

from PyQt5.QtCore import Qt
from PyQt5.QtWidgets import QApplication, QMainWindow, QLabel, QVBoxLayout, QWidget

app = QApplication(sys.argv)
window = QMainWindow()
window.setWindowTitle("XMLB Editor Test")
window.resize(800, 600)
label = QLabel("PyQt5 XMLB Editor - Loaded Successfully!")
label.setAlignment(Qt.AlignCenter)
window.setCentralWidget(label)
window.show()
print("Window shown successfully")
sys.exit(app.exec_())