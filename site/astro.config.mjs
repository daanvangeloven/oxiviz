// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
	site: 'https://oxiviz.digidaan.io',
	integrations: [
		starlight({
			title: 'oxiviz',
			description: 'An offline music visualizer that renders audio files to video.',
			customCss: ['./src/styles/custom.css'],
			sidebar: [
				{
					label: 'Getting started',
					items: [
						{ label: 'Introduction', slug: 'introduction' },
						{ label: 'Installation', slug: 'guides/installation' },
						{ label: 'Quick start', slug: 'guides/quick-start' },
					],
				},
				{
					label: 'Guides',
					items: [{ label: 'Running in a container', slug: 'guides/containers' }],
				},
				{
					label: 'Concepts',
					items: [
						{ label: 'How it works', slug: 'concepts/how-it-works' },
						{ label: 'Effects', slug: 'concepts/effects' },
						{ label: 'Determinism and seeds', slug: 'concepts/determinism' },
					],
				},
				{
					label: 'Reference',
					items: [{ label: 'CLI', slug: 'reference/cli' }],
				},
			],
		}),
	],
});
