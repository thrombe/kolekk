<script lang="ts">

    export let title = "None"
    export let img_source = ""
    export let lazy = false

    let lazy_img_src = ""
    let observer: any = null

    if (lazy) {
        observer = new IntersectionObserver(onIntersect, {rootMargin: '200px'});
    } else {
        lazy_img_src = img_source
    }

    function onIntersect(entries: any) {
        if (!lazy_img_src && entries[0].isIntersecting) {
            lazy_img_src = img_source;
        }
    }

    function lazyLoad(node: any) {
        observer && observer.observe(node);
        return {
            destroy() {
                observer && observer.unobserve(node)
            }
        }
    }

</script>


<dw style="">
    <article use:lazyLoad style="max-width: 200px;">
        <img alt="cover image of: {title}" src={lazy_img_src} style="height: 200px">
    </article>
    {title}
</dw>


<style>
    dw {
        display:flex;
        flex-direction: column;
        align-items: center;
    }
    cl {
        display:flex;
        flex-direction: row;
    }
</style>
