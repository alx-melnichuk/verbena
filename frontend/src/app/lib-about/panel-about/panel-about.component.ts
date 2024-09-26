import { ChangeDetectionStrategy, Component, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TranslateModule, TranslateService } from '@ngx-translate/core';

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
  public title: string = '';
  public label: string = '';
  public frontendItems: string[] = [];
  public backendItem01 = this.appAbout['backend01'] || '';
  public backendItem02: string[] = this.appAbout['backend02'] || [];
  public backendItem03: string[] = this.appAbout['backend03'] || [];
  public linkSwaggerUi = '/swagger-ui/';
  public linkRapidoc = '/rapidoc';
  public linkRedoc = '/redoc';

  public get appAbout(): any {
    return APP_ABOUT || {};
  }
  public set appAbout(value: any) {
  }

  constructor(private translate: TranslateService,) {
    const appName = this.translate.instant('app.name') || '';
    this.title = this.translate.instant('panel-about.title', { appName: appName }) || '';
    this.label = this.translate.instant('panel-about.label', { appName: appName }) || '';
    this.frontendItems = this.translate.instant('panel-about.frontend_items') || [];
  }
  
  // ** Public API **
  
  // ** Private API **

}
