import { ChangeDetectorRef, Directive, ElementRef, HostListener, inject, Inject, Input, OnInit, Optional } from "@angular/core";
import { APP_SIDEBAR_PARENT, SidebarParent } from "./sidebar-parent";

const CN_INTERVAL = 100;

let uniqueIdCounter = 0;

@Directive({
    selector: "[appSidebarHandler]",
    exportAs: "appSidebarHandler",
    standalone: true
})
export class SidebarHandler implements OnInit {
    private elementRef: ElementRef<HTMLElement> = inject<ElementRef<HTMLElement>>(ElementRef);

    @Input()
    public id = `sdbr-hnd-id${uniqueIdCounter++}`;
    @Input("sidebarHandlerOwner")
    public owner: HTMLElement | null = this.elementRef.nativeElement.parentElement;
    @Input("sidebarHandlerIfScroll")
    public overIfScroll: boolean = false;

    private changeDetector: ChangeDetectorRef = inject(ChangeDetectorRef);
    public readonly parent: SidebarParent | null = inject(APP_SIDEBAR_PARENT, { optional: true });

    public isOpen: boolean = false;
    public isOver: boolean = false;

    private timerOpenPanelEvent: any = null;
    private countOfChecks: number = 0;
    private duration: number = 0;  // millisecond (10^-3)
    private count: number = 0;

    // #constructor() {
    // #    this.owner = this.elementRef.nativeElement.parentElement;
    // #}

    @HostListener("click", ["$event"])
    public doResizePanel(event: Event): void {
        event.preventDefault();
        event.stopPropagation();

        this.toggleOpen();
        if (!!this.overIfScroll) {
            this.checkScrollByTimer(this.count);
        }
    }

    ngOnInit(): void {
        const style = window.getComputedStyle(this.elementRef.nativeElement);
        const duratStr = style.getPropertyValue("---sb-durat");
        this.duration = this.getMillisecond(duratStr); // millisecond 10^-3
        this.count = Math.ceil(this.duration / CN_INTERVAL);
    }

    // ** Public API **

    public toggleOpen(): void {
        this.isOpen = !this.isOpen;
        this.parent?.setIsOpen(this.isOpen);
        this.changeDetector.markForCheck();
    }

    public checkScrollByTimer(count: number): void {
        if (this.timerOpenPanelEvent !== null) {
            clearTimeout(this.timerOpenPanelEvent);
        }
        this.countOfChecks = count;
        const isForward = !this.isOver;
        this.checkScrollPanelByTimer(isForward);
    }

    // ** Private API **

    private isScroll(element: HTMLElement | null): boolean {
        return !!element && (element.scrollWidth > element.clientWidth);
    }

    private checkScrollPanelByTimer(isForward: boolean): void {
        const isScrollBar = this.isScroll(this.owner);
        this.isOver = !this.isOver ? this.isOpen && isScrollBar : this.isOpen;
        this.countOfChecks--;
        if (isForward) {
            this.parent?.setIsOver(this.isOver);
        }
        if (this.countOfChecks > 0) {
            this.timerOpenPanelEvent = setTimeout(() => {
                this.timerOpenPanelEvent = null;
                this.checkScrollPanelByTimer(isForward);
            }, CN_INTERVAL);
        } else if (!isForward) {
            this.parent?.setIsOver(this.isOver);
        }
    }

    private getMillisecond(value: string): number {
        let ratio = 1000;
        const idx1 = value.indexOf("ms");
        if (idx1 > -1) {
            value = value.replace("ms", "");
            ratio = 1;
        } else {
            value = value.replace("s", "");
        }
        return parseFloat(value) * ratio;
    }
}
