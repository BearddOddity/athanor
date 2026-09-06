#!/usr/bin/env bun
/**
 * X-Men Legends II XMLB Editor - Bun HTTP server
 * Serves the web editor UI and handles XMLB file operations
 */

import { existsSync, statSync, readFileSync, writeFileSync, readdirSync } from "fs";
import { join, dirname, basename } from "path";

const SHARED_DIR = process.env.XMEN2_SHARE || "D:/re-lab-share/xmen2_mod";
const XMLB_DIR = join(SHARED_DIR, "xmlb_samples");
const EDITOR_DIR = XMLB_DIR;
const PORT = 3456;

// XMLB format constants
const MAGIC = 0x000011B1;
const HEADER_SIZE = 24;
const NODE_SIZE = 32;

class XMLBFile {
  data: Uint8Array;
  path: string;
  magic: number;
  version: number;
  strtabOff: number;
  unk1: number;
  strCount: number;
  unk2: number;
  strings: Map<number, string>;
  stringsRev: Map<string, number>;
  nodes: any[];
  modified: boolean;

  constructor() {
    this.data = new Uint8Array(0);
    this.path = "";
    this.magic = 0;
    this.version = 0;
    this.strtabOff = 0;
    this.unk1 = 0;
    this.strCount = 0;
    this.unk2 = 0;
    this.strings = new Map();
    this.stringsRev = new Map();
    this.nodes = [];
    this.modified = false;
  }

  static load(path: string): XMLBFile {
    const xmlb = new XMLBFile();
    xmlb.path = path;
    const buffer = readFileSync(path);  // Returns Node.js Buffer
    xmlb.data = new Uint8Array(buffer);
    xmlb.magic = xmlb.readUint32(0);
    xmlb.version = xmlb.readUint32(4);
    xmlb.strtabOff = xmlb.readUint32(8);
    xmlb.unk1 = xmlb.readUint32(12);
    xmlb.strCount = xmlb.readUint32(16);
    xmlb.unk2 = xmlb.readUint32(20);
    if (xmlb.magic !== MAGIC) {
      throw new Error(`Invalid magic: 0x${xmlb.magic.toString(16)} (expected 0x11B1)`);
    }
    xmlb.buildStringTable();
    xmlb.parseNodes();
    return xmlb;
  }

  readUint32(offset: number): number {
    const d = this.data;
    return d[offset] | (d[offset + 1] << 8) | (d[offset + 2] << 16) | (d[offset + 3] << 24);
  }

  writeUint32(offset: number, value: number): void {
    const d = this.data;
    d[offset] = value & 0xFF;
    d[offset + 1] = (value >> 8) & 0xFF;
    d[offset + 2] = (value >> 16) & 0xFF;
    d[offset + 3] = (value >>> 24) & 0xFF;
  }

  buildStringTable(): void {
    let pos = this.strtabOff;
    for (let i = 0; i < this.strCount; i++) {
      if (pos >= this.data.length) break;
      if (this.data[pos] === 0) { pos++; continue; }
      let end = pos;
      while (end < this.data.length && this.data[end] !== 0) end++;
      const s = new TextDecoder("iso-8859-1").decode(this.data.slice(pos, end));
      this.strings.set(pos, s);
      this.stringsRev.set(s, pos);
      pos = end + 1;
    }
  }

  parseNodes(): void {
    for (let n = 0; n < this.strCount; n++) {
      const base = 0x18 + n * NODE_SIZE;
      if (base + NODE_SIZE > this.data.length) break;
      const offsets: number[] = [];
      for (let i = 0; i < 8; i++) {
        offsets.push(this.readUint32(base + i * 4));
      }
      const typeOff = offsets[0];
      const nameOff = offsets[1];
      let typeStr = this.strings.get(typeOff);
      if (typeOff === 0xFFFFFFFF) typeStr = null;
      else if (typeStr === undefined) typeStr = `<0x${typeOff.toString(16).padStart(4, '0')}>`;
      let nameStr = this.strings.get(nameOff);
      if (nameOff === 0xFFFFFFFF) nameStr = null;
      else if (nameStr === undefined) nameStr = `<0x${nameOff.toString(16).padStart(4, '0')}>`;
      const props: any[] = [];
      for (let i = 0; i < 3; i++) {
        const keyOff = offsets[2 + i * 2];
        const val = offsets[3 + i * 2];
        if (keyOff === 0xFFFFFFFF) continue;
        let key = this.strings.get(keyOff);
        if (key === undefined) key = `<0x${keyOff.toString(16).padStart(4, '0')}>`;
        let valStr: string | null = null;
        let valRaw: number | null = null;
        if (val === 0xFFFFFFFF) {
          valStr = null;
          valRaw = null;
        } else if (this.strings.has(val)) {
          valStr = this.strings.get(val)!;
        } else {
          valStr = val.toString();
          valRaw = val;
        }
        props.push({ key, value: valStr ?? valRaw, isRaw: valRaw !== null });
      }
      this.nodes.push({
        index: n,
        fileOffset: base,
        type: typeStr,
        name: nameStr,
        props
      });
    }
  }

  toJSON(): any {
    return {
      path: this.path,
      size: this.data.length,
      magic: this.magic,
      version: this.version,
      strtabOff: this.strtabOff,
      strCount: this.strCount,
      unk1: this.unk1,
      unk2: this.unk2,
      strings: Object.fromEntries(this.strings),
      nodes: this.nodes
    };
  }

  save(path?: string): void {
    const savePath = path ?? this.path;
    writeFileSync(savePath, this.data);
    this.path = savePath;
    this.modified = false;
  }
}

// Cache loaded XMLB files
const loadedFiles: Map<string, XMLBFile> = new Map();

function getXMLB(filename: string): XMLBFile | null {
  const cached = loadedFiles.get(filename);
  if (cached) return cached;
  const path = join(XMLB_DIR, filename);
  if (!existsSync(path)) return null;
  try {
    const xmlb = XMLBFile.load(path);
    loadedFiles.set(filename, xmlb);
    return xmlb;
  } catch (e) {
    console.error(`Error loading ${filename}:`, e);
    return null;
  }
}

const server = Bun.serve({
  port: PORT,
  development: true,
  async fetch(req) {
    const url = new URL(req.url);
    const path = url.pathname;
    const method = req.method;

    // CORS headers for development
    const corsHeaders = {
      "Access-Control-Allow-Origin": "*",
      "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
      "Access-Control-Allow-Headers": "Content-Type",
    };

    if (method === "OPTIONS") {
      return new Response(null, { status: 204, headers: corsHeaders });
    }

    try {
      // API: List files
      if (path === "/api/files" && method === "GET") {
        const files: any[] = [];
        try {
          const entries = readdirSync(XMLB_DIR);
          for (const name of entries) {
            if (name.toLowerCase().endsWith(".xmlb")) {
              const fullPath = join(XMLB_DIR, name);
              const stat = statSync(fullPath);
              files.push({ name, size: stat.size });
            }
          }
        } catch (e) {}
        return Response.json({ files }, { headers: corsHeaders });
      }

      // API: Load XMLB file
      const loadMatch = path.match(/^\/api\/load\/(.+)$/);
      if (loadMatch && method === "GET") {
        const filename = decodeURIComponent(loadMatch[1]);
        const xmlb = getXMLB(filename);
        if (!xmlb) {
          return Response.json({ error: "File not found" }, { status: 404, headers: corsHeaders });
        }
        return Response.json(xmlb.toJSON(), { headers: corsHeaders });
      }

      // API: Set property
      const propMatch = path.match(/^\/api\/property\/(.+)\/(\d+)\/(.+)$/);
      if (propMatch && method === "POST") {
        const filename = decodeURIComponent(propMatch[1]);
        const nodeIdx = parseInt(propMatch[2]);
        const propKey = propMatch[3];
        const xmlb = getXMLB(filename);
        if (!xmlb) {
          return Response.json({ error: "File not found" }, { status: 404, headers: corsHeaders });
        }
        const body = await req.json() as any;
        const node = xmlb.nodes[nodeIdx];
        if (!node) {
          return Response.json({ error: "Node not found" }, { status: 404, headers: corsHeaders });
        }
        const prop = node.props.find((p: any) => p.key === propKey);
        if (!prop) {
          return Response.json({ error: "Property not found" }, { status: 404, headers: corsHeaders });
        }
        const value = body.value;
        if (typeof value === "number") {
          prop.value = value;
          prop.isRaw = true;
        } else {
          prop.value = value;
          prop.isRaw = false;
        }
        return Response.json({ ok: true }, { headers: corsHeaders });
      }

      // API: Save
      if (path === "/api/save" && method === "POST") {
        const body = await req.json() as any;
        const filename = body.filename;
        const xmlb = getXMLB(filename);
        if (!xmlb) {
          return Response.json({ error: "File not found" }, { status: 404, headers: corsHeaders });
        }
        xmlb.save();
        return Response.json({ ok: true, path: xmlb.path }, { headers: corsHeaders });
      }

      // API: Deploy
      if (path === "/api/deploy" && method === "POST") {
        const body = await req.json() as any;
        const filename = body.filename;
        const destDir = body.destDir || "D:/My Games/X-Men Legends II Rise of Apocalypse/UI/menus";
        const xmlb = getXMLB(filename);
        if (!xmlb) {
          return Response.json({ error: "File not found" }, { status: 404, headers: corsHeaders });
        }
        const destPath = join(destDir, filename);
        xmlb.save(destPath);
        return Response.json({ ok: true, path: destPath }, { headers: corsHeaders });
      }

      // Serve editor.html at root
      if (path === "/" || path === "/editor") {
        const htmlPath = join(EDITOR_DIR, "editor.html");
        if (existsSync(htmlPath)) {
          return new Response(readFileSync(htmlPath), {
            headers: { ...corsHeaders, "Content-Type": "text/html" }
          });
        }
        return new Response("editor.html not found at " + htmlPath, { status: 404 });
      }

      // Default response
      return Response.json({ error: "Not found", path }, { status: 404, headers: corsHeaders });
    } catch (e: any) {
      console.error("Server error:", e);
      return Response.json({ error: e.message || String(e) }, { status: 500, headers: corsHeaders });
    }
  },
});

console.log(`XMLB Editor server running at http://localhost:${server.port}`);
console.log(`XMLB directory: ${XMLB_DIR}`);
console.log(`Editor UI: http://localhost:${server.port}/editor`);
