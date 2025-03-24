import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PanelChatComponent } from './panel-chat.component';

describe('PanelChatComponent', () => {
  let component: PanelChatComponent;
  let fixture: ComponentFixture<PanelChatComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PanelChatComponent]
    })
    .compileComponents();

    fixture = TestBed.createComponent(PanelChatComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
