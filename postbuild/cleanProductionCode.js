import { glob } from "glob";
import fs from "fs/promises";

function cleanProductionCode(sourceCode) {
  try {
    let lines = sourceCode.split("\n");
    let cleanedLines = [];
    let skipBlock = false;
    let openBraces = 0;

    for (let i = 0; i < lines.length; i++) {
      const currentLine = lines[i];

      // Handle single-line "Remove in production"
      if (currentLine.includes("// CRITICAL: Remove line in production!")) {
        const codePart = currentLine.split("//")[0].trim();
        // If there's code before the comment on the same line, it's a single-line removal
        if (codePart.length > 0) {
          continue;
        }
      }

      // Handle block "Remove in production"
      if (currentLine.includes("// CRITICAL: Remove block in production!")) {
        skipBlock = true;
        // Find start of the block by scanning backward for statement start
        let j = cleanedLines.length - 1;
        while (j >= 0) {
          const line = cleanedLines[j].trim();
          if (line.endsWith(";") || line.endsWith("{") || line.endsWith(")")) {
            break;
          }
          j--;
        }
        cleanedLines = cleanedLines.slice(0, j); // Remove entire block start

        // Scan forward until the block closes
        openBraces = (currentLine.match(/{/g) || []).length - (currentLine.match(/}/g) || []).length;
        while (skipBlock && i < lines.length) {
          i++;
          const line = lines[i];
          openBraces += (line.match(/{/g) || []).length;
          openBraces -= (line.match(/}/g) || []).length;
          if (openBraces <= 0) {
            skipBlock = false;
          }
        }
        continue;
      }

      // Handle "Add in production" comments - moved after removal checks
      if (currentLine.includes("// CRITICAL: Add line in production!")) {
        const parts = currentLine.split("!");
        if (parts.length > 1) {
          const codeToAdd = parts[1].trim();
          if (codeToAdd) {
            cleanedLines.push(currentLine);  // Keep the original line
            cleanedLines.push(codeToAdd);    // Add the extracted code on next line
          }
        }
        continue;
      }

      // Add the line if it wasn't skipped
      cleanedLines.push(currentLine);
    }

    // Clean up trailing empty lines
    while (cleanedLines.length > 0 && cleanedLines[cleanedLines.length - 1].trim() === "") {
      cleanedLines.pop();
    }

    return cleanedLines.join("\n") + "\n";
  } catch (error) {
    throw new Error(`Error cleaning code: ${error.message}`);
  }
}

// Function to process multiple files
async function processFiles(patterns) {
  try {
    const patternArray = Array.isArray(patterns) ? patterns : [patterns];

    const stats = {
      processed: 0,
      failed: 0,
      skipped: 0,
    };

    for (const pattern of patternArray) {
      const files = await glob(pattern, { absolute: true });

      for (const filePath of files) {
        try {
          console.log(`Processing: ${filePath}`);
          const content = await fs.readFile(filePath, "utf8");

          const cleanedContent = cleanProductionCode(content);

          if (content.trim() === cleanedContent.trim()) {
            console.log(`No changes needed for: ${filePath}`);
            stats.skipped++;
            continue;
          }

          const backupPath = `${filePath}.bak`;
          await fs.writeFile(backupPath, content);
          await fs.writeFile(filePath, cleanedContent);

          console.log(`Successfully processed: ${filePath}`);
          stats.processed++;
        } catch (error) {
          console.error(`Failed to process ${filePath}: ${error.message}`);
          stats.failed++;
        }
      }
    }

    console.log("\nProcessing Summary:");
    console.log(`Files processed: ${stats.processed}`);
    console.log(`Files skipped (no changes): ${stats.skipped}`);
    console.log(`Files failed: ${stats.failed}`);

    return stats;
  } catch (error) {
    throw new Error(`Error processing files: ${error.message}`);
  }
}

export { cleanProductionCode, processFiles };