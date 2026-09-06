// ExportXmlbInfo.java
// Ghidra headless post-script: dump all functions and strings to a file
import ghidra.app.script.GhidraScript;
import ghidra.program.model.listing.*;
import ghidra.program.model.address.*;
import ghidra.program.model.mem.*;
import java.io.*;

public class ExportXmlbInfo extends GhidraScript {

    public void run() throws Exception {
        String dllName = currentProgram.getName();
        String outputPath = "D:\\re-lab-share\\xmen2_mod\\xmlb_samples\\ghidra_funcs_" + dllName + ".txt";
        PrintStream ps = new PrintStream(new FileOutputStream(outputPath));
        ps.println("=== " + dllName + " ===");
        ps.println("Image base: 0x" + Long.toHexString(currentProgram.getImageBase().getOffset()));
        ps.println("Function count: " + currentProgram.getFunctionManager().getFunctionCount());
        ps.println();
        FunctionIterator fi = currentProgram.getFunctionManager().getFunctions(true);
        int n = 0;
        while (fi.hasNext()) {
            Function f = fi.next();
            ps.println(String.format("0x%08X  size=%6d  %s",
                f.getEntryPoint().getOffset(),
                f.getBody().getNumAddresses(),
                f.getName()));
            n++;
        }
        ps.println();
        ps.println("=== STRINGS (4+ chars) ===");
        MemoryBlock[] blocks = currentProgram.getMemory().getBlocks();
        for (MemoryBlock b : blocks) {
            if (!b.isInitialized() || !b.isReadable()) continue;
            long s = b.getStart().getOffset();
            long e = b.getEnd().getOffset();
            long cur = s;
            while (cur <= e) {
                StringBuilder sb = new StringBuilder();
                long strStart = cur;
                while (cur <= e) {
                    byte by = currentProgram.getMemory().getByte(cur);
                    if (by == 0) break;
                    if (by < 0x20 || by > 0x7E) break;
                    sb.append((char) by);
                    cur++;
                }
                if (sb.length() >= 4) {
                    ps.println("0x" + Long.toHexString(strStart) + "  " + sb.toString());
                }
                cur++;
            }
        }
        ps.close();
        println("Wrote: " + outputPath);
    }
}