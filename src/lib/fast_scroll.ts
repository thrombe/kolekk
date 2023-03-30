
export const fastScroll = (e: any) => {
    const scroll = (ev: WheelEvent) => {
        e.scrollBy({ left: ev.deltaX*3, top: ev.deltaY*3, behavior: "auto" });
        ev.preventDefault();
    };
    e.addEventListener("wheel", scroll);
    return {
        destroy() {
            e.removeEventListener("wheel", scroll);
        }
    }
};

