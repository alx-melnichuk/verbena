import { ChangeDetectionStrategy, Component, HostBinding, Input, OnChanges, SimpleChanges, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-sidebar',
  exportAs: 'appSidebar',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './sidebar.component.html',
  styleUrls: ['./sidebar.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class SidebarComponent implements OnChanges {
  @Input()
  public isOpen: boolean = false;
  
  @HostBinding('attr.is-open')
  public get attrIsOpen(): boolean | null {
    return this.isOpen ? true : null;
  }

  constructor() {
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (!!changes['isOpen']) {
      console.log(`isOpen: ${this.isOpen}`); // #
    }
  }
}
