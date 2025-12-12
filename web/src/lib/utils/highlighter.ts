import { escapeSvelte } from 'mdsvex';
import { createHighlighter, bundledLanguages, type Highlighter } from 'shiki';

const THEMES = {
	light: 'github-light',
	dark: 'github-dark'
} as const;

const highlighter: Highlighter = await createHighlighter({
	themes: Object.values(THEMES),
	langs: Object.keys(bundledLanguages)
});

export default function highlight(code: string, lang: string = 'text'): string {
	const html = escapeSvelte(
		highlighter.codeToHtml(code, {
			lang: lang in bundledLanguages ? lang : 'text',
			themes: THEMES,
			defaultColor: false
		})
	);
	return `{@html \`${html}\`}`;
}