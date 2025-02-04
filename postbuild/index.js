import { processFiles } from "./cleanProductionCode.js";

// Multiple patterns
await processFiles(["programs/collider-beta/src/**/*.rs"]);
