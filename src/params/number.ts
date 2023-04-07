import type { ParamMatcher } from '@sveltejs/kit';

export const match = ((v: string) => {
    return /^\d+$/.test(v);
}) satisfies ParamMatcher;
