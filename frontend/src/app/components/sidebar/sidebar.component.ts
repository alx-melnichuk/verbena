import {
    ChangeDetectionStrategy, ChangeDetectorRef, Component, EventEmitter, HostBinding, Input, OnChanges, Output, SimpleChanges,
    ViewEncapsulation
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { APP_SIDEBAR_PARENT, SidebarParent } from './sidebar-parent.interface';

let uniqueIdCounter = 0;

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
export class SidebarComponent implements SidebarParent, OnChanges {
    @Input()
    public id = `app-sdbr-${uniqueIdCounter++}`;
    @Input()
    public mode: 'left' | 'right' = 'left';
    @Input()
    public isOpen: boolean = false;
    @Input()
    public isOver: boolean = false;

    @Output()
    readonly changeOpen: EventEmitter<boolean> = new EventEmitter();
    @Output()
    readonly changeOver: EventEmitter<boolean> = new EventEmitter();

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

    constructor(private changeDetector: ChangeDetectorRef) {
    }

    ngOnChanges(changes: SimpleChanges): void {
        if (!!changes['isOpen'] && !!changes['isOpen'].currentValue != !!changes['isOpen'].previousValue) {
            this.changeOpen.emit(this.isOpen);
        }
        if (!!changes['isOver'] && !!changes['isOver'].currentValue != !!changes['isOver'].previousValue) {
            this.changeOver.emit(this.isOver);
        }
    }

    // ** Public API **

    // ** SidebarParent **

    public setIsOpen(isOpen: boolean): void {
        if (this.isOpen != isOpen) {
            this.isOpen = isOpen;
            this.changeDetector.markForCheck();
            this.changeOpen.emit(this.isOpen);
        }
    }

    public setIsOver(isOver: boolean): void {
        if (this.isOver != isOver) {
            this.isOver = isOver;
            this.changeDetector.markForCheck();
            this.changeOver.emit(this.isOver);
        }
    }

    // ** Private API **
}
