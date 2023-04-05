<script lang="ts">
    export let enter_screen: () => void | Promise<void> = async () => {};
    export let leave_screen: () => void | Promise<void> = async () => {};

    export let visible: Boolean | null = null;
    export let root: HTMLElement | null = null;

    const on_intersect = async (entries: any) => {
        if (entries[0].isIntersecting) {
            visible = true;
            await enter_screen();
        } else {
            visible = false;
            await leave_screen();
        }
    };

    function observe(node: any) {
        observer && observer.observe(node);
        return {
            destroy() {
                observer && observer.unobserve(node);
            }
        };
    }

    let observer = new IntersectionObserver(on_intersect, { rootMargin: '200px', root: root });
</script>

<p use:observe />
