import { ChangeDetectionStrategy, Component, HostBinding, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TranslateModule } from '@ngx-translate/core';

declare var APP_ABOUT: any;

@Component({
  selector: 'app-panel-about',
  standalone: true,
  imports: [CommonModule, TranslateModule,],
  templateUrl: './panel-about.component.html',
  styleUrls: ['./panel-about.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PanelAboutComponent {
  public backendItem01 = this.appAbout['backend01'] || '';
  public backendItem02: string[] = this.appAbout['backend02'] || [];
  public backendItem03: string[] = this.appAbout['backend03'] || [];

  public get appAbout(): any {
    return APP_ABOUT || {};
  }
  public set appAbout(value: any) {
  }

  @HostBinding('class.global-scroll')
  public get isGlobalScroll(): boolean { return true; }

  constructor() {
  }
  
  // ** Public API **
  
  // ** Private API **

}
