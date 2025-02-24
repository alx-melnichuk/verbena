import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelLoginComponent } from './panel-login.component';

describe('PanelLoginComponent', () => {
  let component: PanelLoginComponent;
  let fixture: ComponentFixture<PanelLoginComponent>;

  beforeEach(() => {
    TestBed.configureTestingModule({
      imports: [PanelLoginComponent]
    });
    fixture = TestBed.createComponent(PanelLoginComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
