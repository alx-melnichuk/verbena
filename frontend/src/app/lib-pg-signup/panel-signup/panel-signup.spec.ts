import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelSignup } from './panel-signup';

describe('PanelSignup', () => {
  let component: PanelSignup;
  let fixture: ComponentFixture<PanelSignup>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelSignup]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelSignup);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
