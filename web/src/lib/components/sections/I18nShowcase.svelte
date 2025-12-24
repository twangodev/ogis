<script lang="ts">
	import { cubicOut } from 'svelte/easing';
	import type { TransitionConfig } from 'svelte/transition';
	import ImagePreview from '$lib/components/playground/ImagePreview.svelte';

	type I18nCard = {
		title: string;
		description: string;
		subtitle: string;
		language: string;
		template: string;
	};

	// Custom transition with fly + rotate
	function flyRotate(
		_node: Element,
		{ x = 0, rotate = 0, delay = 0, duration = 400, easing = cubicOut }: {
			x?: number;
			rotate?: number;
			delay?: number;
			duration?: number;
			easing?: (t: number) => number;
		} = {}
	): TransitionConfig {
		return {
			delay,
			duration,
			easing,
			css: (t) => {
				const transform = `translateX(${(1 - t) * x}px) rotate(${(1 - t) * rotate}deg)`;
				return `opacity: ${t}; transform: ${transform};`;
			}
		};
	}

	// Animation config - alternating from left/right with rotation
	const getAnimation = (index: number) => ({
		x: index % 2 === 0 ? -150 : 150,
		rotate: index % 2 === 0 ? -15 : 15,
		delay: 500 + index * 80,
		duration: 700
	});

	let sectionRef = $state<HTMLElement | null>(null);
	let isVisible = $state(false);

	// Trigger animation when section comes into view
	$effect(() => {
		if (!sectionRef) return;

		const observer = new IntersectionObserver(
			(entries) => {
				if (entries[0].isIntersecting) {
					isVisible = true;
					observer.disconnect();
				}
			},
			{ threshold: 0.15 }
		);

		observer.observe(sectionRef);

		return () => observer.disconnect();
	});

	const cards: I18nCard[] = [
		{
			title: '美しいOG画像を生成',
			description: 'Rustで動作する高速で無料の画像生成サービス',
			subtitle: '日本語',
			language: 'Japanese',
			template: 'gradient-ember'
		},
		{
			title: '生成精美的社交预览图',
			description: '由 Rust 驱动的快速免费图像生成',
			subtitle: '简体中文',
			language: 'Chinese',
			template: 'gradient-ruby'
		},
		{
			title: '아름다운 OG 이미지 생성',
			description: 'Rust로 구동되는 빠르고 무료인 이미지',
			subtitle: '한국어',
			language: 'Korean',
			template: 'gradient-sapphire'
		},
		{
			title: 'सुंदर OG छवियाँ बनाएं',
			description: 'Rust द्वारा संचालित तेज़ मुफ्त छवि निर्माण',
			subtitle: 'हिन्दी',
			language: 'Hindi',
			template: 'gradient-peach'
		},
		{
			title: 'สร้างภาพ OG ที่สวยงาม',
			description: 'บริการสร้างภาพที่รวดเร็วและฟรี',
			subtitle: 'ภาษาไทย',
			language: 'Thai',
			template: 'gradient-jade'
		},
		{
			title: 'إنشاء صور معاينة جميلة',
			description: 'خدمة إنشاء صور سريعة ومجانية مدعومة بـ Rust',
			subtitle: 'العربية',
			language: 'Arabic',
			template: 'gradient-galaxy'
		},
		{
			title: 'יצירת תמונות OG יפות',
			description: 'שירות יצירת תמונות מהיר וחינמי מבוסס Rust',
			subtitle: 'עברית',
			language: 'Hebrew',
			template: 'gradient-cobalt'
		},
		{
			title: 'Создавайте красивые изображения',
			description: 'Быстрая и бесплатная генерация на Rust',
			subtitle: 'Русский',
			language: 'Russian',
			template: 'gradient-midnight'
		},
		{
			title: 'Créez de belles images OG',
			description: "Génération d'images rapide propulsée par Rust",
			subtitle: 'Français',
			language: 'French',
			template: 'gradient-lavender'
		}
	];

	function buildImageUrl(card: I18nCard): string {
		const params = new URLSearchParams();
		params.set('title', card.title);
		params.set('description', card.description);
		params.set('subtitle', card.subtitle);
		params.set('template', card.template);
		// Optimize for display: WebP at 40% scale with quality 80
		params.set('format', 'webp');
		params.set('scale', '0.4');
		params.set('quality', '80');
		return `https://img.ogis.dev?${params.toString()}`;
	}
</script>

<section bind:this={sectionRef}>
	<div class="py-12 md:py-24">
		<div class="relative mx-auto max-w-7xl px-6">
			<div class="mb-10 text-left">
				<h2 class="text-3xl font-semibold md:text-4xl">Global Language Support</h2>
				<p class="mt-4 max-w-2xl text-lg text-muted-foreground">
					Full Unicode support with automatic font detection. Japanese, Chinese, Korean, Arabic,
					Hebrew, Thai, Hindi, and more — all rendered beautifully.
				</p>
			</div>

			<div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
				{#each cards as card, index (card.language)}
					{#if isVisible}
						{@const anim = getAnimation(index)}
						<div
							class="group"
							in:flyRotate={{ x: anim.x, rotate: anim.rotate, delay: anim.delay, duration: anim.duration }}
						>
							<div
								class="overflow-hidden rounded-xl border border-border shadow-md transition-shadow duration-300 group-hover:shadow-xl"
							>
								<ImagePreview
									src={buildImageUrl(card)}
									alt="{card.language} example"
									aspectRatio="aspect-[1.91]"
									class="transition-transform duration-300 group-hover:scale-[1.02]"
								/>
							</div>
							<p class="mt-2 text-sm text-muted-foreground">{card.language}</p>
						</div>
					{/if}
				{/each}
			</div>
		</div>
	</div>
</section>
