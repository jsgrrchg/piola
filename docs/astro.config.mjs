// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import piolaGrammar from './src/grammars/piola.tmLanguage.json';

export default defineConfig({
  integrations: [
    starlight({
      title: 'Piola',
      description: 'Un lenguaje de programación chileno, simple y directo.',
      favicon: './src/assets/logo.svg',
      logo: {
        src: './src/assets/logo.svg',  
      },

     social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/cuervolu/piola' }],

      customCss: ['./src/styles/custom.css'],


      expressiveCode: {
        themes: ['one-dark-pro'],
 		shiki: {                 
          langs: [piolaGrammar],  
        },
      },

      sidebar: [
        {
          label: '¿Qué es Piola?',
          items: [
            { label: 'Introducción', link: '/' },
            { label: 'Instalación', link: '/instalacion' },
            { label: 'Filosofía', link: '/filosofia' },
          ],
        },
        {
          label: 'El lenguaje',
          items: [
            { label: 'Variables', link: '/lenguaje/variables' },
            { label: 'Funciones', link: '/lenguaje/funciones' },
            { label: 'Control de flujo', link: '/lenguaje/control-flujo' },
            { label: 'Tipos de datos', link: '/lenguaje/tipos' },
            { label: 'Errores', link: '/lenguaje/errores' },
          ],
        },
        // {
        //   label: 'Internals',
        //   collapsed: true,
        //   items: [
        //     { label: 'Pipeline', link: '/internals/pipeline' },
        //     { label: 'Lexer', link: '/internals/lexer' },
        //     { label: 'Parser', link: '/internals/parser' },
        //     { label: 'Intérprete', link: '/internals/interprete' },
        //   ],
        // },
      ],
    }),
  ],

  site: 'https://cuervolu.github.io',
  base: '/piola',
});