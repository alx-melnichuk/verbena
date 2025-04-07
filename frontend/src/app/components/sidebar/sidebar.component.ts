import { ChangeDetectionStrategy, Component, EventEmitter, HostBinding, Input, Output, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
    selector: 'app-sidebar',
    exportAs: 'appSidebar',
    standalone: true,
    imports: [CommonModule],
    templateUrl: './sidebar.component.html',
    styleUrl: './sidebar.component.scss',
    encapsulation: ViewEncapsulation.None,
    changeDetection: ChangeDetectionStrategy.OnPush
})
export class SidebarComponent {
    // @Input()
    // @HostBinding('attr.is-above-parent')
    // public isAboveParent: boolean = false;
    @Input()
    public isOpen: boolean = false;
    @Input()
    public isOver: boolean = false;

    // @Output()
    // readonly clickByVeil: EventEmitter<boolean> = new EventEmitter();

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

    constructor() {
    }

    // public doClickByVeil(): void {
    //     this.clickByVeil.emit(true);
    // }
}
