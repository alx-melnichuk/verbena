import { AfterViewInit, Directive, ElementRef, EventEmitter, HostListener, inject, Output } from "@angular/core";

const TIMEOUT_VALUE = 50;

@Directive({
    selector: "[appScrollable]",
    exportAs: "appScrollable",
    standalone: true,
})
export class Scrollable implements AfterViewInit {
    private readonly elementRef = inject<ElementRef<HTMLElement>>(ElementRef);

    @Output("appScrollable")
    readonly scrollable: EventEmitter<{ isScrollX: boolean, isScrollY: boolean }> = new EventEmitter();

    private clientHeight: number = 0;
    private clientWidth: number = 0;
    private isScrollY: boolean = false;
    private isScrollX: boolean = false;

    private timerTimeout: number | null = null;

    @HostListener("window:resize")
    onResize(): void {
        if (this.timerTimeout != null) {
            return;
        }
        this.checkExistScroll();
    }

    ngAfterViewInit(): void {
        this.checkExistScroll();
    }

    private checkExistScroll(): void {
        requestAnimationFrame(() => {
            const elem = this.elementRef.nativeElement;

            const isScrollX = this.clientWidth != elem.clientWidth && elem.clientWidth < elem.scrollWidth;
            this.clientWidth = elem.clientWidth;

            const isScrollY = this.clientHeight != elem.clientHeight && elem.clientHeight < elem.scrollHeight;
            this.clientHeight = elem.clientHeight;

            if (this.isScrollX != isScrollX || this.isScrollY != isScrollY) {
                this.isScrollX = isScrollX;
                this.isScrollY = isScrollY;
                this.scrollable.emit({ isScrollX, isScrollY });

                if (!!this.timerTimeout) {
                    window.clearTimeout(this.timerTimeout);
                }
                this.timerTimeout = window.setTimeout(() => {
                    this.timerTimeout = null;
                }, TIMEOUT_VALUE);
            }
        });
    }
}
