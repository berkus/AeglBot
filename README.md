LFG bot

## Добавьте себя

с помощью команды /psn:

**/psn mypsnid**

например

`/psn PUSHISTIK_79` 

Если вы уже зареганы, бот скажет об этом. Без регистрации нельзя добавляться в активности ЛФГ, поэтому это обязательно сделать в первую очередь.


## Чтобы посмотреть список текущих планов 

**/list**

бот покажет будущие или недавно начавшиеся активности с другими гардианами. В списке есть цифровые айди - с их помощью можно прицепляться.

Например

`/list` 

бот ответит


    Planned activities:
    
    3: dozniak (@berkus), Hunny_Lang (@Hunny_Lang), Gera2159 (@Gera2159) going to Wrath of the Machine normal
    on 10/28/16 at 9:30 PM (starts in 4 days 8 hours 16 minutes)
    Enter /join3 to join this group.


## Присоединиться или отказаться

Набрав **/join3** можно будет подключиться к этой активности. Если вы передумали, наберите **/cancel3** и бот уберет вас из списка.

Если вы были последним в списке, то после команды /cancel вся активность удаляется.

Цифры айди, естественно, каждый раз разные, смотрите в выводе команды list, какие именно цифры использовать.

## Новая активность

Чтобы создать активность используйте команду /lfg

**/activities**

выдаст список коротких названий, которые можно использовать для создания.

Затем, например

`/lfg wotmn friday 21:30` 

создаст поход на аксиса на нормале в эту пятницу, в 21:30 московского времени.


Спецификация времени работает в свободной форме, понимаемой библотекой natty. Поэкспериментировать, как она поймет ваше предложение времени можно вот тут http://natty.joestelmach.com/try.jsp

Время по-умолчанию стоит Московское!

