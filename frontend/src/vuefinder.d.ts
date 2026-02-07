// src/global-components.d.ts
import type { VueFinder } from 'vuefinder';

declare module 'vue' {
    export interface GlobalComponents {
        VueFinder: typeof VueFinder;
    }
}