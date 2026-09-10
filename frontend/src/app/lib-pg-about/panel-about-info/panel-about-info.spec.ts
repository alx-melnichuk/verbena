import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelAboutInfo } from './panel-about-info';

describe('PanelAboutInfo', () => {
  let component: PanelAboutInfo;
  let fixture: ComponentFixture<PanelAboutInfo>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelAboutInfo]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelAboutInfo);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
