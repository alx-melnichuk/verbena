import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelAbout } from './panel-about';

describe('PanelAbout', () => {
  let component: PanelAbout;
  let fixture: ComponentFixture<PanelAbout>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelAbout]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelAbout);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
