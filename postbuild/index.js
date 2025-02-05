import { glob } from "glob";
import fs from "fs/promises";
import { processFiles } from "./cleanProductionCode.js";

async function replaceCargoTestFilesContent(pattern) {
    try {
        const files = await glob(pattern, { absolute: true });

        const newTestContent = `#[tokio::test]
        async fn test_full_collider_flow() {
            println!("Note: No testing available for production branch. Please switch to 'localnet' branch for testing");
        }
        `;

        let processed = 0;
        let failed = 0;

        for (const filePath of files) {
            if (filePath.endsWith(".rs")) {
                try {
                    console.log(`Replacing content of: ${filePath}`);
                    await fs.writeFile(filePath, newTestContent);
                    processed++;
                } catch (error) {
                    console.error(`Failed to process ${filePath}: ${error.message}`);
                    failed++;
                }
            }
        }

        console.log("\nCargo Replacement Summary:");
        console.log(`Files successfully processed: ${processed}`);
        console.log(`Files failed: ${failed}`);

    } catch (error) {
        console.error(`Error replacing test file content: ${error.message}`);
    }
}

async function replaceAnchorTestFilesContent(pattern) {
    try {
        const files = await glob(pattern, { absolute: true });

        const newTestContent = `describe("collider-beta", () => {
            console.log("Note: No testing available for production branch. Please switch to 'localnet' branch for testing");
        });
        `;

        let processed = 0;
        let failed = 0;

        for (const filePath of files) {
            if (filePath.endsWith(".ts")) {
                try {
                    console.log(`Replacing content of: ${filePath}`);
                    await fs.writeFile(filePath, newTestContent);
                    processed++;
                } catch (error) {
                    console.error(`Failed to process ${filePath}: ${error.message}`);
                    failed++;
                }
            }
        }

        console.log("\nAnchor Replacement Summary:");
        console.log(`Files successfully processed: ${processed}`);
        console.log(`Files failed: ${failed}`);

    } catch (error) {
        console.error(`Error replacing test file content: ${error.message}`);
    }
}

async function main() {
    // Process test files with the specific pattern
    await processFiles(["programs/collider-beta/src/**/*.rs"]);

    // Replace test file content for all Cargo test files
    await replaceCargoTestFilesContent("programs/collider-beta/tests/*.rs");

    // Replace test file content for all Anchor test files
    await replaceAnchorTestFilesContent("tests/*.ts");
}

main().catch(console.error);

