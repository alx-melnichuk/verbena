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
    @Input()
    public isOpen: boolean = false;
    @Input()
    public isVeilByClosing: boolean = false;

    @Output()
    readonly clickByVeil: EventEmitter<boolean> = new EventEmitter();

    @HostBinding('attr.is-open')
    public get attrIsOpen(): boolean | null {
        return this.isOpen ? true : null;
    }

    @HostBinding('class.sb-relative')
    public get classIsRelativeVal(): boolean {
        return this.isVeilByClosing;
    }

    constructor() {
    }

    public doClickByVeil(): void {
        this.clickByVeil.emit(true);
    }
}
