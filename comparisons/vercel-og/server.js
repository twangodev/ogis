import { createServer } from 'node:http';
import { ImageResponse } from '@vercel/og';

const PORT = process.env.PORT || 3000;

const server = createServer(async (req, res) => {
  const url = new URL(req.url, `http://localhost:${PORT}`);

  // Health check endpoint
  if (url.pathname === '/health') {
    res.writeHead(200, { 'Content-Type': 'text/plain' });
    res.end('OK');
    return;
  }

  // Only handle root path
  if (url.pathname !== '/') {
    res.writeHead(404);
    res.end('Not Found');
    return;
  }

  const title = url.searchParams.get('title') || 'Default Title';
  const description = url.searchParams.get('description') || 'Default description';

  try {
    // Minimal template - exactly matches ogis minimal.svg layout
    // Title at y=250, Description at y=300 (from SVG coordinates)
    const response = new ImageResponse(
      {
        type: 'div',
        props: {
          style: {
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            width: '100%',
            height: '100%',
            backgroundColor: '#ffffff',
            position: 'relative',
          },
          children: [
            {
              type: 'div',
              props: {
                style: {
                  position: 'absolute',
                  top: '250px',
                  left: '0',
                  right: '0',
                  fontSize: '80px',
                  fontWeight: 'bold',
                  color: '#111827',
                  textAlign: 'center',
                  fontFamily: 'sans-serif',
                },
                children: title,
              },
            },
            {
              type: 'div',
              props: {
                style: {
                  position: 'absolute',
                  top: '350px',
                  left: '0',
                  right: '0',
                  fontSize: '28px',
                  color: '#9ca3af',
                  textAlign: 'center',
                  fontFamily: 'sans-serif',
                },
                children: description,
              },
            },
          ],
        },
      },
      {
        width: 1200,
        height: 630,
      }
    );

    const buffer = await response.arrayBuffer();

    res.writeHead(200, {
      'Content-Type': 'image/png',
      'Content-Length': buffer.byteLength,
    });
    res.end(Buffer.from(buffer));
  } catch (error) {
    console.error('Error generating image:', error);
    res.writeHead(500, { 'Content-Type': 'text/plain' });
    res.end(`Error: ${error.message}`);
  }
});

server.listen(PORT, () => {
  console.log(`Vercel OG benchmark server running on port ${PORT}`);
});