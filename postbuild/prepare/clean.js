import { glob } from "glob";
import fs from "fs/promises";

function clean(sourceCode) {
  try {
    let lines = sourceCode.split("\n");
    let cleanedLines = [];
    let skipBlock = false;
    let inTestBlock = false;
    let openBraces = 0;

    for (let i = 0; i < lines.length; i++) {
      const currentLine = lines[i];

      // Handle "Add line in production!"
      if (currentLine.includes("// CRITICAL: Add line in production!")) {
        const parts = currentLine.split("!");
        if (parts.length > 1) {
          const codeToAdd = parts[1].trim();
          if (codeToAdd) {
            cleanedLines.push("\t" + codeToAdd);
          }
        }
        continue;
      }

      // Handle single-line "Remove line in production"
      if (currentLine.includes("// CRITICAL: Remove line in production!")) {
        continue;
      }

      // Handle block "Remove block in production!"
      if (currentLine.includes("// CRITICAL: Remove block in production!")) {
        skipBlock = true;

        // Find start of the block
        let j = cleanedLines.length - 1;
        while (j >= 0) {
          const line = cleanedLines[j].trim();
          if (line.endsWith(";") || line.endsWith("{") || line.endsWith(")")) {
            break;
          }
          j--;
        }
        cleanedLines = cleanedLines.slice(0, j);

        // Skip until block closes
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

      // Handle #[cfg(test)] and subsequent test code block
      if (currentLine.trim().startsWith("#[cfg(test)]")) {
        inTestBlock = true;
        continue;
      }

      if (inTestBlock) {
        continue;
      }

      // Add the line if it wasn't skipped
      cleanedLines.push(currentLine);
    }

    // Clean up trailing empty lines
    while (cleanedLines.length > 0 && cleanedLines[cleanedLines.length - 1].trim() === "") {
      cleanedLines.pop();
    }

    // Final pass to remove lingering comments
    cleanedLines = cleanedLines.filter(
      (line) => !line.trim().startsWith("// CRITICAL: Add line in production!")
    );

    return cleanedLines.join("\n") + "\n";
  } catch (error) {
    throw new Error(`Error cleaning code: ${error.message}`);
  }
}

// Function to process multiple files
async function process(patterns) {
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

          const cleanedContent = clean(content);

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

    console.log("\nProduction Preparation Summary:");
    console.log(`Files processed: ${stats.processed}`);
    console.log(`Files skipped (no changes): ${stats.skipped}`);
    console.log(`Files failed: ${stats.failed}`);

    return stats;
  } catch (error) {
    throw new Error(`Error processing files: ${error.message}`);
  }
}

export { clean, process };
