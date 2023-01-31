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


<card-div draggable="true">
    <image-div use:lazyLoad>
        <img alt="cover image of: {title}" src={lazy_img_src}>
    </image-div>
    <span class="title">
        {title}
    </span>
</card-div>


<style>
    image-div {
        width: 100%;
        /* height: 100%; */
        height: calc(100% - 2.7ch);
        overflow: hidden;
    }
    
    image-div img {
        margin-left: 6px;
        margin-right: 6px;
        margin-top: 6px;
        border-radius: 15px;

        /* contain image */
        width: calc(100% - 12px);
        /* width: calc(100% - padding-left - padding-right); */
        height: calc(100% - 6px);
        object-fit: cover;
    }

    .title {
        padding-left: 8px;
        padding-right: 8px;
        width: calc(100% - 16px);
        height: 2.7ch;

        text-align: center;
        
        text-overflow: ellipsis;
        overflow: hidden;
        white-space: nowrap;
    }

    card-div {
        display:flex;
        flex-direction: column;
        align-items: center;
        color: rgb(179, 179, 179);

        /* - 2px for border */
        width: calc(25vw - 2px);
        /* height: 40vw; */
        aspect-ratio: 0.6;
        background-color: blueviolet;

        border: 1px solid;
        border-radius: 15px;
        border-color: red;
    }
</style>
