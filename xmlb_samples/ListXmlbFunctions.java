// ListXmlbFunctions.java
// Ghidra headless script: list all functions in libIGAttrs.dll / libIGGui.dll
// and save to a text file for analysis.

import ghidra.app.script.GhidraScript;
import ghidra.program.model.listing.*;
import ghidra.program.model.address.*;
import java.io.*;

public class ListXmlbFunctions extends GhidraScript {

    @Override
    public void run() throws Exception {
        String dllName = getCurrentProgram().getName();
        String outputPath = "D:\\re-lab-share\\xmen2_mod\\xmlb_samples\\ghidra_funcs_" + dllName + ".txt";
        PrintWriter pw = new PrintWriter(new FileWriter(outputPath));

        pw.println("=== " + dllName + " ===");
        pw.println("Image base: 0x" + Long.toHexString(getCurrentProgram().getImageBase().getOffset()));
        pw.println("Entry point: 0x" + Long.toHexString(getCurrentProgram().getEntryPoint().getOffset()));
        pw.println("Function count: " + getCurrentProgram().getFunctionManager().getFunctionCount());
        pw.println();

        // List all functions with their addresses and sizes
        FunctionIterator funcs = getCurrentProgram().getFunctionManager().getFunctions(true);
        int count = 0;
        while (funcs.hasNext()) {
            Function f = funcs.next();
            Address addr = f.getEntryPoint();
            long size = f.getBody().getNumAddresses();
            pw.println(String.format("0x%08X  size=%6d  %s", 
                addr.getOffset(), size, f.getName()));
            count++;
        }
        pw.println();
        pw.println("Total functions listed: " + count);

        // List all strings
        pw.println();
        pw.println("=== STRINGS ===");
        MemoryIterator memIter = getCurrentProgram().getMemory().getBlocks();
        while (memIter.hasNext()) {
            MemoryBlock block = memIter.next();
            if (block.isInitialized() && block.isReadable()) {
                Address start = block.getStart();
                Address end = block.getEnd();
                long cur = start.getOffset();
                while (cur <= end.getOffset()) {
                    StringBuilder sb = new StringBuilder();
                    long strStart = cur;
                    while (cur <= end.getOffset()) {
                        byte b = getByte(cur);
                        if (b == 0) break;
                        if (b < 0x20 || b > 0x7E) break;
                        sb.append((char) b);
                        cur++;
                    }
                    if (sb.length() >= 4) {
                        pw.println("0x" + Long.toHexString(strStart) + "  " + sb.toString());
                    }
                    cur++;
                }
            }
        }

        pw.close();
        println("Exported to: " + outputPath);
    }
}