import { ImageResponse } from 'next/og';
import { NextRequest } from 'next/server';

export const runtime = 'edge';

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);

    const title = searchParams.get('title') || 'Open Graph Images';
    const description = searchParams.get('description') || 'A fast, free, and beautiful platform for open graph image generation';
    const subtitle = searchParams.get('subtitle') || 'img.ogis.dev';
    const imageUrl = searchParams.get('image');

    return new ImageResponse(
      (
        <div
          style={{
            height: '100%',
            width: '100%',
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'flex-start',
            justifyContent: 'center',
            backgroundColor: '#fff',
            padding: '80px',
          }}
        >
          <div
            style={{
              display: 'flex',
              flexDirection: 'column',
              gap: '20px',
            }}
          >
            {subtitle && (
              <div
                style={{
                  fontSize: 24,
                  color: '#666',
                  marginBottom: 10,
                }}
              >
                {subtitle}
              </div>
            )}
            <div
              style={{
                fontSize: 72,
                fontWeight: 'bold',
                color: '#000',
                lineHeight: 1.2,
                maxWidth: '900px',
              }}
            >
              {title}
            </div>
            {description && (
              <div
                style={{
                  fontSize: 32,
                  color: '#555',
                  maxWidth: '900px',
                }}
              >
                {description}
              </div>
            )}
          </div>
          {imageUrl && (
            <div
              style={{
                position: 'absolute',
                top: 80,
                right: 80,
                display: 'flex',
              }}
            >
              <img
                src={imageUrl}
                width={200}
                height={200}
                alt="Logo"
                style={{
                  objectFit: 'contain',
                }}
              />
            </div>
          )}
        </div>
      ),
      {
        width: 1200,
        height: 630,
      }
    );
  } catch (e: any) {
    console.error(`Failed to generate image: ${e.message}`);
    return new Response(`Failed to generate image: ${e.message}`, {
      status: 500,
    });
  }
}