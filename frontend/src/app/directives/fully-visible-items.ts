import { AfterViewInit, Directive, ElementRef, EventEmitter, HostListener, inject, Output } from "@angular/core";

const TIMEOUT_VALUE = 100;

@Directive({
    selector: "[appFullyVisibleItems]",
    exportAs: "appFullyVisibleItems",
    standalone: true,
})
export class FullyVisibleItems implements AfterViewInit {
    private readonly elementRef = inject<ElementRef<HTMLElement>>(ElementRef);

    @Output()
    readonly visibleItems: EventEmitter<number> = new EventEmitter();

    private countByHeight: number = 0;
    private countByWidth: number = 0;
    private timerTimeout: number | undefined;

    @HostListener("window:resize")
    onResize(): void {
        if (!!this.timerTimeout) {
            return;
        }
        this.timerTimeout = window.setTimeout(() => {
            this.timerTimeout = undefined;
        }, TIMEOUT_VALUE);
        this.updateVisibleItems();
    }

    ngAfterViewInit(): void {
        requestAnimationFrame(() => {
            this.updateVisibleItems();
        });
    }

    // ** Public API **

    // ** Private API **

    private updateVisibleItems(): void {
        const container = this.elementRef.nativeElement;
        const availableWidth = container.clientWidth;
        // const elements = Array.from(container.querySelectorAll<HTMLElement>("app-menu-bar"));
        const elements = Array.from(container.children) as HTMLElement[];

        let countByHeight = 0;
        let countByWidth = 0;
        let usedByHeight = 0;
        let usedWidth = 0;
        for (let idx = 0; idx < elements.length; idx++) {
            const width = elements[idx].offsetWidth;
            if (usedWidth + width <= availableWidth) {
                usedWidth += width;
                countByWidth++;
            } else {
                break;
            }
            const height = elements[idx].offsetHeight;
            if (usedByHeight + height <= availableWidth) {
                usedByHeight += height;
                countByHeight++;
            } else {
                break;
            }
        }

        if (this.countByWidth != countByWidth) {
            this.visibleItems.emit(this.countByWidth = countByWidth);
        } else if (this.countByHeight != countByHeight) {
            this.visibleItems.emit(this.countByHeight = countByHeight);
        }
    }
}
