import { escapeSvelte } from 'mdsvex';
import { createHighlighter, bundledLanguages } from 'shiki';

const THEMES = {
	light: 'github-light',
	dark: 'github-dark'
};

const highlighter = await createHighlighter({
	themes: Object.values(THEMES),
	langs: Object.keys(bundledLanguages)
});

export default function highlight(code, lang = 'text') {
	const html = escapeSvelte(
		highlighter.codeToHtml(code, {
			lang: lang in bundledLanguages ? lang : 'text',
			themes: THEMES,
			defaultColor: false
		})
	);
	return `{@html \`${html}\`}`;
}
