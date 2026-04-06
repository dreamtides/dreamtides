import type { Plugin } from "vite";
import { mkdirSync, appendFileSync } from "fs";
import { join } from "path";

const LOG_DIR = join(__dirname, "..", "..", ".logs", "battle_prototype");

export default function battleLogger(): Plugin {
  return {
    name: "battle-logger",
    configureServer(server) {
      mkdirSync(LOG_DIR, { recursive: true });

      server.middlewares.use("/__log", (req, res) => {
        if (req.method !== "POST") {
          res.statusCode = 405;
          res.end();
          return;
        }

        let body = "";
        req.on("data", (chunk: Buffer) => {
          body += chunk.toString();
        });
        req.on("end", () => {
          try {
            const entry = JSON.parse(body);
            const sessionId: string = entry.session_id ?? "unknown";
            const file = join(LOG_DIR, `${sessionId}.jsonl`);
            appendFileSync(file, JSON.stringify(entry) + "\n");
            res.statusCode = 200;
            res.end("ok");
          } catch {
            res.statusCode = 400;
            res.end("bad json");
          }
        });
      });
    },
  };
}
