import { ChangeDetectionStrategy, Component, ViewEncapsulation } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TranslateModule, TranslateService } from '@ngx-translate/core';

declare var APP_ABOUT: any;

@Component({
  selector: 'app-about',
  standalone: true,
  imports: [CommonModule, TranslateModule,],
  templateUrl: './about.component.html',
  styleUrls: ['./about.component.scss'],
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class AboutComponent {
  public title: string = '';
  public label: string = '';
  public frontendItems: string[] = [];
  public backendItem01 = APP_ABOUT['backend01'] || '';
  public backendItem02: string[] = APP_ABOUT['backend02'] || [];
  public backendItem03: string[] = APP_ABOUT['backend03'] || [];
  public linkSwaggerUi = '/swagger-ui/';
  public linkRapidoc = '/rapidoc';
  public linkRedoc = '/redoc';

  constructor(private translate: TranslateService,) {
    console.log('PgAboutComponent();');
    const appName = this.translate.instant('app_name') || '';
    this.title = this.translate.instant('about.title', { app_name: appName }) || '';
    this.label = this.translate.instant('about.label', { app_name: appName }) || '';
    this.frontendItems = this.translate.instant('about.frontend_items') || [];
  }
  
  // ** Public API **
  
  // ** Private API **
  
}
