import { file } from 'bun';

const PORT = 3000;

console.log(`Starting Aura Prototype dev server on http://localhost:${PORT}`);

Bun.serve({
  port: PORT,
  async fetch(req) {
    const url = new URL(req.url);
    let pathname = url.pathname;

    // Default to index.html
    if (pathname === '/') {
      pathname = '/index.html';
    }

    // Resolve file path
    const filePath = pathname.startsWith('/') ? pathname.slice(1) : pathname;
    const fullPath = `${import.meta.dir}/${filePath}`;

    try {
      const f = file(fullPath);
      const exists = await f.exists();

      if (!exists) {
        // Try .tsx extension for TypeScript files
        const tsxPath = `${import.meta.dir}/${filePath.replace(/\.js$/, '.tsx')}`;
        const tsxFile = file(tsxPath);
        if (await tsxFile.exists()) {
          // Build and serve TypeScript
          const result = await Bun.build({
            entrypoints: [tsxPath],
            format: 'esm',
            minify: false,
          });

          if (result.success && result.outputs.length > 0) {
            const text = await result.outputs[0].text();
            return new Response(text, {
              headers: { 'Content-Type': 'application/javascript' },
            });
          }
        }

        return new Response('Not Found', { status: 404 });
      }

      // Get content type
      const ext = pathname.split('.').pop()?.toLowerCase();
      const contentTypes: Record<string, string> = {
        html: 'text/html',
        css: 'text/css',
        js: 'application/javascript',
        ts: 'application/javascript',
        tsx: 'application/javascript',
        json: 'application/json',
        png: 'image/png',
        svg: 'image/svg+xml',
      };

      // Handle TypeScript/TSX files - transpile them
      if (ext === 'ts' || ext === 'tsx') {
        const result = await Bun.build({
          entrypoints: [fullPath],
          format: 'esm',
          minify: false,
        });

        if (result.success && result.outputs.length > 0) {
          const text = await result.outputs[0].text();
          return new Response(text, {
            headers: { 'Content-Type': 'application/javascript' },
          });
        }

        return new Response('Build Error', { status: 500 });
      }

      return new Response(f, {
        headers: {
          'Content-Type': contentTypes[ext || ''] || 'application/octet-stream',
        },
      });
    } catch (error) {
      console.error('Error serving file:', error);
      return new Response('Internal Server Error', { status: 500 });
    }
  },
});
