import {
    ChangeDetectionStrategy, ChangeDetectorRef, Component, EventEmitter, HostBinding, Inject, Input,
    ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { APP_SIDEBAR_PARENT, SidebarParent } from './sidebar-parent.interface';

@Component({
    selector: 'app-sidebar',
    exportAs: 'appSidebar',
    standalone: true,
    imports: [CommonModule],
    templateUrl: './sidebar.component.html',
    styleUrl: './sidebar.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush,
    providers: [{ provide: APP_SIDEBAR_PARENT, useExisting: SidebarComponent }],
})
export class SidebarComponent implements SidebarParent {
    @Input()
    public mode: 'left' | 'right' = 'left';
    @Input()
    public isOpen: boolean = false;
    @Input()
    public isOver: boolean = false;

    // @Output()
    // readonly clickByVeil: EventEmitter<boolean> = new EventEmitter();

    @HostBinding('attr.is-left')
    public get attrIsLeft(): string | null {
        return this.mode == 'left' ? '' : null;
    }

    @HostBinding('attr.is-right')
    public get attrIsRight(): string | null {
        return this.mode == 'right' ? '' : null;
    }

    @HostBinding('attr.is-open')
    public get attrIsOpen(): string | null {
        return this.isOpen ? '' : null;
    }

    @HostBinding('attr.is-over')
    public get attrIsOver(): string | null {
        return this.isOver ? '' : null;
    }

    // @HostBinding('class.sb-relative')
    // public get classIsRelativeVal(): boolean {
    //     return this.isVeilByClosing;
    // }

    constructor(
        private changeDetector: ChangeDetectorRef,
    ) {
        // console.log(`Sidebar()`);
    }

    // ** Public API **
    // public doClickByVeil(): void {
    //     this.clickByVeil.emit(true);
    // }

    // ** SidebarParent **

    public setIsOpen(isOpen: boolean): void {
        if (this.isOpen != isOpen) {
            this.isOpen = isOpen;
            this.changeDetector.markForCheck();
        }
    }

    public setIsOver(isOver: boolean): void {
        if (this.isOver != isOver) {
            this.isOver = isOver;
            this.changeDetector.markForCheck();
        }
    }

    // ** **
}
